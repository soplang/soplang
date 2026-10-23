//! Validate the recorded schema before changing any application table.
use super::*;

const JOURNAL: &str = "soplang_schema";
const MAX_SCHEMA: usize = 1024 * 1024;

enum Previous {
    Empty,
    Legacy(String),
    Schema(Schema),
}

fn invalid() -> SoplangError {
    data_error("Diiwaanka guurintu ma saxna; hubi diiwaanka ka hor guurinta.")
}
fn checked_hash(text: &str) -> Result<&str, SoplangError> {
    if text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(text)
    } else {
        Err(invalid())
    }
}
fn decode(
    hash: Option<String>,
    snapshot: Option<(String, String)>,
) -> Result<Previous, SoplangError> {
    let Some(hash) = hash else {
        return if snapshot.is_none() {
            Ok(Previous::Empty)
        } else {
            Err(invalid())
        };
    };
    checked_hash(&hash)?;
    let Some((snapshot_hash, text)) = snapshot else {
        return Ok(Previous::Legacy(hash));
    };
    if snapshot_hash != hash || text.len() > MAX_SCHEMA {
        return Err(invalid());
    }
    let schema = parse_schema(&text).map_err(|_| invalid())?;
    if schema.fingerprint() != hash {
        return Err(invalid());
    }
    Ok(Previous::Schema(schema))
}
fn limited(kind: DriverKind, fields: &str, table: &str, limit: usize) -> String {
    let table = sql::identifier(kind, table);
    if matches!(kind, DriverKind::SqlServer) {
        format!("SELECT TOP {limit} {fields} FROM {table}")
    } else {
        format!("SELECT {fields} FROM {table} LIMIT {limit}")
    }
}
fn text_at(row: &SqlRow, index: usize) -> Result<String, SoplangError> {
    match row.get(index) {
        Some(SqlValue::Text(s)) => Ok(s.clone()),
        _ => Err(invalid()),
    }
}
fn load(db: &mut Database) -> Result<Previous, SoplangError> {
    if let Conn::Mongo(mongo) = db.current() {
        let Some(record) = mongo
            .database
            .collection::<Document>("soplang_guuri")
            .find_one(doc! {"_id": "schema"})
            .run()
            .map_err(driver_error)?
        else {
            return Ok(Previous::Empty);
        };
        let hash = record.get_str("sha256").map_err(|_| invalid())?.to_string();
        let snapshot = match record.get("qaab") {
            None => None,
            Some(Bson::String(text)) => Some((hash.clone(), text.clone())),
            _ => return Err(invalid()),
        };
        return decode(Some(hash), snapshot);
    }
    let hash = if existing_columns(db, "soplang_guuri")?.is_empty() {
        None
    } else {
        let rows = query(db, &limited(db.kind, "sha256", "soplang_guuri", 2))?;
        match rows.len() {
            0 => None,
            1 => Some(text_at(&rows[0], 0)?),
            _ => return Err(invalid()),
        }
    };
    let snapshot = if existing_columns(db, JOURNAL)?.is_empty() {
        None
    } else {
        let rows = query(db, &limited(db.kind, "sha256, qaab", JOURNAL, 2))?;
        if rows.len() != 1 {
            return Err(invalid());
        }
        Some((text_at(&rows[0], 0)?, text_at(&rows[0], 1)?))
    };
    decode(hash, snapshot)
}

/// Return tables that must be empty before adding a required column.
fn successor<'a>(old: &Schema, next: &'a Schema) -> Result<Vec<&'a schema::Table>, SoplangError> {
    let mut empty = Vec::new();
    for table in &old.tables {
        let target = next.table(&table.name).ok_or_else(|| {
            data_error(format!(
                "Guurintu ma saari karto jadwalka '{}'.",
                table.name
            ))
        })?;
        for column in &table.columns {
            if target.columns.iter().find(|c| c.name == column.name) != Some(column) {
                return Err(data_error(format!(
                    "Guurintu ma beddeli karto ama saari karto tiirka '{}.{}'.",
                    table.name, column.name
                )));
            }
        }
        for column in &target.columns {
            if table.columns.iter().all(|c| c.name != column.name) {
                if column.key {
                    return Err(data_error("Guurintu ma beddeli karto furaha jadwalka."));
                }
                if !column.optional && !empty.iter().any(|t: &&schema::Table| t.name == target.name)
                {
                    empty.push(target);
                }
            }
        }
    }
    Ok(empty)
}
fn columns(db: &mut Database, table: &str) -> Result<Option<Vec<String>>, SoplangError> {
    if let Conn::Mongo(mongo) = db.current() {
        let exists = mongo
            .database
            .list_collection_names()
            .run()
            .map_err(driver_error)?
            .iter()
            .any(|name| name == table);
        return Ok(exists.then(Vec::new));
    }
    let names = existing_columns(db, table)?;
    Ok((!names.is_empty()).then_some(names))
}
fn not_empty(db: &mut Database, table: &str) -> Result<bool, SoplangError> {
    if let Conn::Mongo(mongo) = db.current() {
        return mongo
            .database
            .collection::<Document>(table)
            .find_one(doc! {})
            .run()
            .map(|row| row.is_some())
            .map_err(driver_error);
    }
    query(db, &limited(db.kind, "1", table, 1)).map(|rows| !rows.is_empty())
}

pub(super) fn preflight(db: &mut Database, next: &Schema) -> Result<(), SoplangError> {
    if next.source().len() > MAX_SCHEMA {
        return Err(data_error(
            "Qaabka xogtu wuu ka weyn yahay xadka guurinta ee 1 MiB.",
        ));
    }
    let old = match load(db)? {
        Previous::Empty => None,
        Previous::Legacy(hash) => {
            if hash != next.fingerprint() {
                return Err(data_error("Guurintii hore waxay kaydisay summad keliya. Marka hore guuri schema-kii hore ee isla summadda leh, dabadeedna qaabka cusub."));
            }
            Some(next.clone())
        }
        Previous::Schema(schema) => Some(schema),
    };
    let require_empty = if let Some(old) = &old {
        successor(old, next)?
    } else {
        Vec::new()
    };
    // Check every recorded table and every proposed addition before any DDL.
    for table in &next.tables {
        let existing = columns(db, &table.name)?;
        if let Some(previous) = old.as_ref().and_then(|old| old.table(&table.name)) {
            let names = existing.ok_or_else(|| {
                data_error(format!(
                    "Jadwalkii '{}' ee diiwaanka ku jiray waa maqan yahay.",
                    table.name
                ))
            })?;
            if !matches!(db.kind, DriverKind::Mongo)
                && (names.len() != previous.columns.len()
                    || previous.columns.iter().any(|c| !names.contains(&c.name)))
            {
                return Err(data_error(format!(
                    "Tiirarka jadwalka '{}' ma waafaqsana diiwaanka guurinta.",
                    table.name
                )));
            }
        } else if existing.is_some() {
            return Err(data_error(format!("Jadwalka '{}' wuu jiraa, balse kuma jiro diiwaanka guurinta. Si toos ah looguma dari karo.", table.name)));
        }
    }
    if let Conn::Mongo(mongo) = db.current() {
        let existing = mongo
            .database
            .list_collection_names()
            .run()
            .map_err(driver_error)?;
        for table in &next.tables {
            if existing.contains(&table.name) {
                let invalid_row = mongo
                    .database
                    .collection::<Document>(&table.name)
                    .find_one(doc! { "$nor": [mongo_validator(table)] })
                    .projection(doc! { "_id": 1 })
                    .run()
                    .map_err(driver_error)?;
                if invalid_row.is_some() {
                    return Err(data_error(format!(
                        "Xogta ku jirta jadwalka '{}' ma waafaqsana qaabka cusub.",
                        table.name
                    )));
                }
            }
        }
    }
    for table in require_empty {
        if not_empty(db, &table.name)? {
            return Err(data_error(format!("Tiir qasab ah laguma dari karo jadwalka '{}' inta xog ku jirto; marka hore ku dar tiir ikhtiyaari ah.", table.name)));
        }
    }
    Ok(())
}

pub(super) fn record(db: &mut Database, schema: &Schema) -> Result<(), SoplangError> {
    let hash = schema.fingerprint();
    let source = schema.source();
    if let Conn::Mongo(mongo) = db.current() {
        mongo
            .database
            .collection::<Document>("soplang_guuri")
            .update_one(
                doc! {"_id": "schema"},
                doc! {"$set": {"sha256": hash, "qaab": source}},
            )
            .upsert(true)
            .run()
            .map_err(driver_error)?;
        return Ok(());
    }
    let create = match db.kind {
        DriverKind::SqlServer => format!("IF OBJECT_ID(N'{JOURNAL}', N'U') IS NULL CREATE TABLE {JOURNAL} (sha256 NVARCHAR(128) NOT NULL, qaab NVARCHAR(MAX) NOT NULL);"),
        DriverKind::Mysql | DriverKind::Mariadb => format!("CREATE TABLE IF NOT EXISTS {JOURNAL} (sha256 VARCHAR(128) NOT NULL, qaab LONGTEXT NOT NULL);"),
        _ => format!("CREATE TABLE IF NOT EXISTS {JOURNAL} (sha256 TEXT NOT NULL, qaab TEXT NOT NULL);"),
    };
    exec(db, &create)?;
    exec(db, &migration_table(db.kind))?;
    exec(db, &format!("DELETE FROM {JOURNAL}"))?;
    exec(db, "DELETE FROM soplang_guuri")?;
    // The hash is restricted to lowercase hex; schema text is always a bound value.
    exec(
        db,
        &format!("INSERT INTO soplang_guuri (sha256) VALUES ('{hash}')"),
    )?;
    let placeholders = match db.kind {
        DriverKind::Postgres | DriverKind::Cockroach => "$1, $2",
        DriverKind::SqlServer => "@P1, @P2",
        _ => "?, ?",
    };
    exec_statement(
        db,
        &Statement {
            text: format!("INSERT INTO {JOURNAL} (sha256, qaab) VALUES ({placeholders})"),
            params: vec![Param::text(hash), Param::text(source)],
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshots_round_trip_and_reject_mismatched_or_malformed_metadata() {
        let schema = parse_schema("jadwal t {\n id abn furaha\n magac qoraal?\n}\n").unwrap();
        assert_eq!(parse_schema(&schema.source()).unwrap(), schema);
        let hash = schema.fingerprint();
        assert!(decode(Some(hash.clone()), Some((hash.clone(), schema.source()))).is_ok());
        assert!(decode(Some("x".into()), None).is_err());
        assert!(decode(None, Some((hash.clone(), schema.source()))).is_err());
        assert!(decode(Some(hash.clone()), Some((hash, "invalid".into()))).is_err());
    }
}
