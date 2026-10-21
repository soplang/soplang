//! `schema.sop` tables for the Soplang data layer.

use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub tables: Vec<Table>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub ty: ColumnType,
    pub optional: bool,
    pub key: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ColumnType {
    Int,
    Text,
    Float,
    Bool,
}

impl Schema {
    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|table| table.name == name)
    }

    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        for table in &self.tables {
            hasher.update(table.name.as_bytes());
            hasher.update([0]);
            for column in &table.columns {
                hasher.update(column.name.as_bytes());
                hasher.update([
                    column.ty as u8,
                    u8::from(column.optional),
                    u8::from(column.key),
                ]);
            }
        }
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

impl ColumnType {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "abn" => Some(Self::Int),
            "qoraal" => Some(Self::Text),
            "jajab" => Some(Self::Float),
            "bool" => Some(Self::Bool),
            _ => None,
        }
    }
}

pub fn parse_schema(text: &str) -> Result<Schema, String> {
    let mut tables = Vec::new();
    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'));
    while let Some(line) = lines.next() {
        let Some(name) = line.strip_prefix("jadwal ") else {
            return Err(format!("Safka schema.sop ma saxna: {line}"));
        };
        let name = name.trim().trim_end_matches('{').trim();
        if !ident(name) {
            return Err(format!("Magaca jadwalka '{name}' ma saxna."));
        }
        if !line.contains('{') {
            let Some(open) = lines.next() else {
                return Err("Jadwalka ma furmin.".into());
            };
            if open != "{" {
                return Err("Jadwalka waa inuu ku furmo {.".into());
            }
        }
        let mut columns = Vec::new();
        loop {
            let Some(line) = lines.next() else {
                return Err(format!("Jadwalka '{name}' ma xidhmin."));
            };
            if line == "}" {
                break;
            }
            let column = parse_column(line)?;
            if columns
                .iter()
                .any(|existing: &Column| existing.name == column.name)
            {
                return Err(format!("Tiirka '{}' waa la soo celiyey.", column.name));
            }
            columns.push(column);
        }
        if columns.iter().filter(|column| column.key).count() != 1 {
            return Err(format!(
                "Jadwalka '{name}' wuxuu u baahan yahay hal furaha abn."
            ));
        }
        if name.starts_with("soplang_") {
            return Err("Magacyada ku bilaabma soplang_ waxaa loo reebay nidaamka.".into());
        }
        if tables.iter().any(|table: &Table| table.name == name) {
            return Err(format!("Jadwalka '{name}' waa la soo celiyey."));
        }
        tables.push(Table {
            name: name.to_string(),
            columns,
        });
    }
    if tables.is_empty() {
        return Err("schema.sop ma leh jadwal.".into());
    }
    Ok(Schema { tables })
}

fn parse_column(line: &str) -> Result<Column, String> {
    let mut parts = line.split_whitespace();
    let name = parts
        .next()
        .ok_or_else(|| "Tiirka schema.sop ma saxna.".to_string())?;
    let ty = parts
        .next()
        .ok_or_else(|| format!("Nooca tiirka '{name}' waa ka maqan yahay."))?;
    if !ident(name) {
        return Err(format!("Magaca tiirka '{name}' ma saxna."));
    }
    let (ty, optional) = if let Some(base) = ty.strip_suffix('?') {
        (base, true)
    } else {
        (ty, false)
    };
    let ty = ColumnType::parse(ty).ok_or_else(|| format!("Nooca '{ty}' lama aqoonsan."))?;
    let mut key = false;
    for flag in parts {
        if flag == "furaha" {
            if key || ty != ColumnType::Int || optional {
                return Err(format!("Furaha '{name}' waa inuu noqdaa abn hal mar."));
            }
            key = true;
        } else {
            return Err(format!("Calaamadda '{flag}' lama aqoonsan."));
        }
    }
    Ok(Column {
        name: name.to_string(),
        ty,
        optional,
        key,
    })
}

pub fn ident(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(ch) if ch == '_' || ch.is_ascii_lowercase())
        && chars.all(|ch| ch == '_' || ch.is_ascii_lowercase() || ch.is_ascii_digit())
        && name.len() <= 64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_table_and_rejects_two_keys() {
        let schema =
            parse_schema("jadwal isticmaale {\n  id abn furaha\n  magac qoraal\n}\n").unwrap();
        assert_eq!(schema.tables[0].columns.len(), 2);
        assert!(parse_schema("jadwal t {\n  a abn furaha\n  b abn furaha\n}\n").is_err());
    }
}
