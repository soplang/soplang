//! Schema, migrations, pools, and real database connectors.
//!
//! SQLite is embedded. PostgreSQL and CockroachDB share the Postgres wire
//! protocol. MySQL and MariaDB share the MySQL protocol. MongoDB and SQL Server
//! use their own clients. Each connector executes statements through that
//! client; none of them return canned rows.

mod schema;

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;

use mongodb::bson::{doc, Bson, Document};
use mongodb::sync::Client as MongoClient;
use mysql::prelude::Queryable;
use postgres::NoTls;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

pub use schema::{ident, parse_schema, ColumnType, Schema};

use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};
use crate::runtime::value::Value;

static NEXT_ID: AtomicI64 = AtomicI64::new(1);
static DATABASES: Mutex<Option<HashMap<i64, Database>>> = Mutex::new(None);

struct Database {
    kind: DriverKind,
    schema: Option<Schema>,
    conns: Vec<Conn>,
    cursor: usize,
    pinned: Option<usize>,
}

impl Database {
    fn current(&mut self) -> &mut Conn {
        let index = if let Some(index) = self.pinned {
            index
        } else {
            let index = self.cursor % self.conns.len().max(1);
            self.cursor = self.cursor.wrapping_add(1);
            index
        };
        &mut self.conns[index]
    }
}

#[derive(Clone, Copy)]
enum DriverKind {
    Sqlite,
    Postgres,
    Cockroach,
    Mysql,
    Mariadb,
    SqlServer,
    Mongo,
}

enum Conn {
    Sqlite(rusqlite::Connection),
    Postgres(postgres::Client),
    Mysql(mysql::Conn),
    Mongo(MongoHandle),
    SqlServer(SqlServer),
}

struct MongoHandle {
    database: mongodb::sync::Database,
    client: MongoClient,
    session: Option<mongodb::sync::ClientSession>,
}

struct SqlServer {
    runtime: tokio::runtime::Runtime,
    client: tiberius::Client<tokio_util::compat::Compat<TcpStream>>,
}

#[derive(Clone, Debug)]
enum SqlValue {
    Null,
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
}

pub fn open(kind: &str, target: &str, pool: usize) -> Result<Value, SoplangError> {
    let kind = DriverKind::parse(kind)?;
    let pool = pool.clamp(1, 8);
    let mut conns = Vec::with_capacity(pool);
    for _ in 0..pool {
        conns.push(connect(kind, target)?);
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let mut databases = DATABASES
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    databases.get_or_insert_with(HashMap::new).insert(
        id,
        Database {
            kind,
            schema: None,
            conns,
            cursor: 0,
            pinned: None,
        },
    );
    Ok(handle(id, kind.name()))
}

pub fn load_schema(id: i64, text: &str) -> Result<Value, SoplangError> {
    let schema = parse_schema(text).map_err(data_error)?;
    with_db(id, |db| {
        db.schema = Some(schema);
        Ok(Value::Null)
    })
}

pub fn migrate(id: i64) -> Result<Value, SoplangError> {
    with_db(id, |db| {
        let schema = db
            .schema
            .clone()
            .ok_or_else(|| data_error("Schema lama akhriyin."))?;
        let result = apply_migration(db, &schema);
        db.pinned = None;
        result?;
        Ok(Value::Null)
    })
}

pub fn insert(id: i64, table: &str, value: &Value) -> Result<Value, SoplangError> {
    with_db(id, |db| {
        let table = require_table(db, table)?;
        let fields = object_fields(value)?;
        reject_unknown(&table, &fields)?;
        match db.current() {
            Conn::Mongo(mongo) => mongo_insert(mongo, &table.name, &fields),
            _ => {
                let sql = insert_sql(db.kind, &table, &fields);
                let count = exec(db, &sql)?;
                Ok(Value::Int(count as i64))
            }
        }
    })
}

pub fn find(id: i64, table: &str, filter: Option<&Value>) -> Result<Value, SoplangError> {
    with_db(id, |db| {
        let table = require_table(db, table)?;
        let filter = match filter {
            Some(value) => object_fields(value)?,
            None => HashMap::new(),
        };
        reject_unknown(&table, &filter)?;
        match db.current() {
            Conn::Mongo(mongo) => mongo_find(mongo, &table.name, &filter),
            _ => {
                let sql = select_sql(db.kind, &table.name, &filter);
                Ok(rows_to_value(&query(db, &sql)?))
            }
        }
    })
}

pub fn update(
    id: i64,
    table: &str,
    filter: &Value,
    changes: &Value,
) -> Result<Value, SoplangError> {
    with_db(id, |db| {
        let table = require_table(db, table)?;
        let filter = object_fields(filter)?;
        let changes = object_fields(changes)?;
        reject_unknown(&table, &filter)?;
        reject_unknown(&table, &changes)?;
        match db.current() {
            Conn::Mongo(mongo) => mongo_update(mongo, &table.name, &filter, &changes),
            _ => Ok(Value::Int(
                exec(db, &update_sql(db.kind, &table.name, &filter, &changes))? as i64,
            )),
        }
    })
}

pub fn delete(id: i64, table: &str, filter: &Value) -> Result<Value, SoplangError> {
    with_db(id, |db| {
        let table = require_table(db, table)?;
        let filter = object_fields(filter)?;
        reject_unknown(&table, &filter)?;
        match db.current() {
            Conn::Mongo(mongo) => mongo_delete(mongo, &table.name, &filter),
            _ => Ok(Value::Int(
                exec(db, &delete_sql(db.kind, &table.name, &filter))? as i64,
            )),
        }
    })
}

pub fn transaction<T>(
    id: i64,
    body: impl FnOnce() -> Result<T, SoplangError>,
) -> Result<T, SoplangError> {
    with_db(id, |db| {
        if db.pinned.is_some() {
            return Err(data_error("Macaamil ayaa horay u furan."));
        }
        begin(db)
    })?;
    let result = body();
    let finished = with_db(id, |db| {
        let outcome = match &result {
            Ok(_) => commit(db),
            Err(_) => rollback(db),
        };
        db.pinned = None;
        outcome
    });
    finished?;
    result
}

pub fn capabilities(kind: &str) -> Result<Value, SoplangError> {
    let kind = DriverKind::parse(kind)?;
    let mongo = matches!(kind, DriverKind::Mongo);
    Ok(object(&[
        ("nooc", Value::Str(kind.name().into())),
        ("abuur", Value::Bool(true)),
        ("geli", Value::Bool(true)),
        ("raadi", Value::Bool(true)),
        ("beddel", Value::Bool(true)),
        ("tirtir", Value::Bool(true)),
        ("guuri", Value::Bool(true)),
        ("hawl", Value::Bool(!mongo)),
        ("macaamil", Value::Bool(true)),
        ("wadar", Value::Int(8)),
    ]))
}

impl DriverKind {
    fn parse(text: &str) -> Result<Self, SoplangError> {
        match text {
            "sqlite" => Ok(Self::Sqlite),
            "postgres" | "postgresql" => Ok(Self::Postgres),
            "cockroach" => Ok(Self::Cockroach),
            "mysql" => Ok(Self::Mysql),
            "mariadb" => Ok(Self::Mariadb),
            "sqlserver" => Ok(Self::SqlServer),
            "mongo" | "mongodb" => Ok(Self::Mongo),
            _ => Err(data_error("Nooca xogta waa inuu noqdaa sqlite, postgres, cockroach, mysql, mariadb, sqlserver, ama mongo.")),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::Cockroach => "cockroach",
            Self::Mysql => "mysql",
            Self::Mariadb => "mariadb",
            Self::SqlServer => "sqlserver",
            Self::Mongo => "mongo",
        }
    }
}

fn connect(kind: DriverKind, target: &str) -> Result<Conn, SoplangError> {
    match kind {
        DriverKind::Sqlite => {
            let conn = rusqlite::Connection::open(target)
                .map_err(|error| data_error(error.to_string()))?;
            conn.busy_timeout(Duration::from_secs(5))
                .map_err(|error| data_error(error.to_string()))?;
            conn.execute_batch("PRAGMA foreign_keys = ON;")
                .map_err(|error| data_error(error.to_string()))?;
            Ok(Conn::Sqlite(conn))
        }
        DriverKind::Postgres | DriverKind::Cockroach => {
            let client = postgres::Client::connect(target, NoTls)
                .map_err(|error| data_error(error.to_string()))?;
            Ok(Conn::Postgres(client))
        }
        DriverKind::Mysql | DriverKind::Mariadb => {
            let conn = mysql::Conn::new(target).map_err(|error| data_error(error.to_string()))?;
            Ok(Conn::Mysql(conn))
        }
        DriverKind::Mongo => {
            let client =
                MongoClient::with_uri_str(target).map_err(|error| data_error(error.to_string()))?;
            let name = mongodb_database(target)?;
            Ok(Conn::Mongo(MongoHandle {
                database: client.database(&name),
                client,
                session: None,
            }))
        }
        DriverKind::SqlServer => Ok(Conn::SqlServer(sql_server(target)?)),
    }
}

fn sql_server(target: &str) -> Result<SqlServer, SoplangError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| data_error(error.to_string()))?;
    let client = runtime.block_on(async {
        let config = tiberius::Config::from_ado_string(target)
            .map_err(|error| data_error(error.to_string()))?;
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(|error| data_error(error.to_string()))?;
        tcp.set_nodelay(true)
            .map_err(|error| data_error(error.to_string()))?;
        tiberius::Client::connect(config, tcp.compat_write())
            .await
            .map_err(|error| data_error(error.to_string()))
    })?;
    Ok(SqlServer { runtime, client })
}

fn mongodb_database(target: &str) -> Result<String, SoplangError> {
    let without = target.split('?').next().unwrap_or(target);
    let name = without.rsplit('/').next().unwrap_or("").trim();
    if name.is_empty() || name.contains(':') {
        return Err(data_error(
            "Cinwaanka MongoDB waa inuu sheegaa magaca xogta, sida mongodb://127.0.0.1/soplang.",
        ));
    }
    Ok(name.to_string())
}

fn apply_migration(db: &mut Database, schema: &Schema) -> Result<(), SoplangError> {
    db.pinned = Some(0);
    if matches!(db.current(), Conn::Mongo(_)) {
        let Conn::Mongo(mongo) = db.current() else {
            return Err(data_error("Xiriirka MongoDB lama helin."));
        };
        for table in &schema.tables {
            mongo
                .database
                .collection::<Document>(&table.name)
                .insert_one(doc! {})
                .run()
                .ok();
            let _ = mongo
                .database
                .collection::<Document>(&table.name)
                .delete_one(doc! {})
                .run();
        }
        let meta = mongo.database.collection::<Document>("soplang_guuri");
        let _ = meta.delete_many(doc! {}).run();
        meta.insert_one(doc! { "sha256": schema.fingerprint() })
            .run()
            .map_err(|error| data_error(error.to_string()))?;
        return Ok(());
    }
    exec(db, &migration_table(db.kind))?;
    for table in &schema.tables {
        exec(db, &create_table(db.kind, table))?;
        let existing = existing_columns(db, &table.name)?;
        for column in &table.columns {
            if !existing.iter().any(|name| name == &column.name) && !column.key {
                exec(db, &add_column(db.kind, &table.name, column))?;
            }
        }
    }
    let hash = schema.fingerprint();
    let _ = exec(db, "DELETE FROM soplang_guuri");
    exec(
        db,
        &format!("INSERT INTO soplang_guuri (sha256) VALUES ('{hash}')"),
    )?;
    Ok(())
}

fn migration_table(kind: DriverKind) -> String {
    match kind {
        DriverKind::SqlServer => "IF OBJECT_ID(N'soplang_guuri', N'U') IS NULL CREATE TABLE soplang_guuri (sha256 NVARCHAR(128) NOT NULL);".into(),
        _ => "CREATE TABLE IF NOT EXISTS soplang_guuri (sha256 TEXT NOT NULL);".into(),
    }
}

fn create_table(kind: DriverKind, table: &schema::Table) -> String {
    let columns = table
        .columns
        .iter()
        .map(|column| column_sql(kind, column))
        .collect::<Vec<_>>()
        .join(", ");
    match kind {
        DriverKind::SqlServer => format!(
            "IF OBJECT_ID(N'{}', N'U') IS NULL CREATE TABLE {} ({columns});",
            table.name, table.name
        ),
        _ => format!("CREATE TABLE IF NOT EXISTS {} ({columns});", table.name),
    }
}

fn column_sql(kind: DriverKind, column: &schema::Column) -> String {
    if column.key {
        return match kind {
            DriverKind::Sqlite => format!("{} INTEGER PRIMARY KEY AUTOINCREMENT", column.name),
            DriverKind::Postgres | DriverKind::Cockroach => format!(
                "{} BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY",
                column.name
            ),
            DriverKind::Mysql | DriverKind::Mariadb => {
                format!("{} BIGINT PRIMARY KEY AUTO_INCREMENT", column.name)
            }
            DriverKind::SqlServer => format!("{} BIGINT IDENTITY(1,1) PRIMARY KEY", column.name),
            DriverKind::Mongo => column.name.clone(),
        };
    }
    let ty = match (kind, column.ty) {
        (_, ColumnType::Int) => "BIGINT",
        (DriverKind::SqlServer, ColumnType::Text) => "NVARCHAR(MAX)",
        (_, ColumnType::Text) => "TEXT",
        (_, ColumnType::Float) => "DOUBLE PRECISION",
        (DriverKind::Mysql | DriverKind::Mariadb, ColumnType::Bool) => "TINYINT",
        (DriverKind::SqlServer, ColumnType::Bool) => "BIT",
        (_, ColumnType::Bool) => "BOOLEAN",
    };
    let ty = if matches!(kind, DriverKind::Mysql | DriverKind::Mariadb)
        && column.ty == ColumnType::Float
    {
        "DOUBLE"
    } else {
        ty
    };
    format!("{} {ty}", column.name)
}

fn add_column(kind: DriverKind, table: &str, column: &schema::Column) -> String {
    format!(
        "ALTER TABLE {table} ADD {} {}",
        if matches!(kind, DriverKind::Postgres | DriverKind::Cockroach) {
            "COLUMN"
        } else {
            ""
        },
        column_sql(kind, column)
    )
    .replace("  ", " ")
}

fn existing_columns(db: &mut Database, table: &str) -> Result<Vec<String>, SoplangError> {
    let sql = match db.kind {
        DriverKind::Sqlite => format!("PRAGMA table_info({table})"),
        DriverKind::SqlServer => format!(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = '{table}'"
        ),
        _ => format!(
            "SELECT column_name FROM information_schema.columns WHERE table_name = '{table}'"
        ),
    };
    let rows = query(db, &sql)?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let index = if matches!(db.kind, DriverKind::Sqlite) {
                1
            } else {
                0
            };
            row.get(index).and_then(|value| match value {
                SqlValue::Text(text) => Some(text.clone()),
                _ => None,
            })
        })
        .collect())
}

fn insert_sql(
    kind: DriverKind,
    table: &schema::Table,
    fields: &HashMap<String, SqlValue>,
) -> String {
    let mut names = Vec::new();
    let mut values = Vec::new();
    for column in &table.columns {
        if column.key && !fields.contains_key(&column.name) {
            continue;
        }
        if let Some(value) = fields.get(&column.name) {
            names.push(column.name.as_str());
            values.push(literal(kind, value));
        }
    }
    format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table.name,
        names.join(", "),
        values.join(", ")
    )
}

fn select_sql(kind: DriverKind, table: &str, filter: &HashMap<String, SqlValue>) -> String {
    let where_sql = where_clause(kind, filter);
    if matches!(kind, DriverKind::SqlServer) {
        format!("SELECT TOP 1000 * FROM {table}{where_sql}")
    } else {
        format!("SELECT * FROM {table}{where_sql} LIMIT 1000")
    }
}

fn update_sql(
    kind: DriverKind,
    table: &str,
    filter: &HashMap<String, SqlValue>,
    changes: &HashMap<String, SqlValue>,
) -> String {
    let set = changes
        .iter()
        .map(|(name, value)| format!("{name} = {}", literal(kind, value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("UPDATE {table} SET {set}{}", where_clause(kind, filter))
}

fn delete_sql(kind: DriverKind, table: &str, filter: &HashMap<String, SqlValue>) -> String {
    format!("DELETE FROM {table}{}", where_clause(kind, filter))
}

fn where_clause(kind: DriverKind, filter: &HashMap<String, SqlValue>) -> String {
    if filter.is_empty() {
        return String::new();
    }
    let parts = filter
        .iter()
        .map(|(name, value)| format!("{name} = {}", literal(kind, value)))
        .collect::<Vec<_>>()
        .join(" AND ");
    format!(" WHERE {parts}")
}

fn literal(kind: DriverKind, value: &SqlValue) -> String {
    match value {
        SqlValue::Null => "NULL".into(),
        SqlValue::Int(value) => value.to_string(),
        SqlValue::Float(value) => {
            if value.is_finite() {
                value.to_string()
            } else {
                "NULL".into()
            }
        }
        SqlValue::Bool(true) => match kind {
            DriverKind::Mysql
            | DriverKind::Mariadb
            | DriverKind::SqlServer
            | DriverKind::Sqlite => "1".into(),
            _ => "TRUE".into(),
        },
        SqlValue::Bool(false) => match kind {
            DriverKind::Mysql
            | DriverKind::Mariadb
            | DriverKind::SqlServer
            | DriverKind::Sqlite => "0".into(),
            _ => "FALSE".into(),
        },
        SqlValue::Text(text) => format!("'{}'", text.replace('\'', "''")),
    }
}

fn begin(db: &mut Database) -> Result<(), SoplangError> {
    let index = db.cursor % db.conns.len().max(1);
    db.pinned = Some(index);
    let result = if matches!(db.kind, DriverKind::Mongo) {
        mongo_begin(db)
    } else {
        let sql = match db.kind {
            DriverKind::Mysql | DriverKind::Mariadb => "START TRANSACTION",
            DriverKind::SqlServer => "BEGIN TRANSACTION",
            _ => "BEGIN",
        };
        exec(db, sql).map(|_| ())
    };
    if result.is_err() {
        db.pinned = None;
    }
    result
}

fn commit(db: &mut Database) -> Result<(), SoplangError> {
    if matches!(db.kind, DriverKind::Mongo) {
        return mongo_finish(db, true);
    }
    exec(db, "COMMIT")?;
    Ok(())
}

fn rollback(db: &mut Database) -> Result<(), SoplangError> {
    if matches!(db.kind, DriverKind::Mongo) {
        return mongo_finish(db, false);
    }
    let _ = exec(db, "ROLLBACK");
    Ok(())
}

fn mongo_begin(db: &mut Database) -> Result<(), SoplangError> {
    let Conn::Mongo(mongo) = db.current() else {
        return Err(data_error("Xiriirka MongoDB lama helin."));
    };
    let mut session = mongo
        .client
        .start_session()
        .run()
        .map_err(|error| data_error(error.to_string()))?;
    session
        .start_transaction()
        .run()
        .map_err(|error| data_error(error.to_string()))?;
    mongo.session = Some(session);
    Ok(())
}

fn mongo_finish(db: &mut Database, commit: bool) -> Result<(), SoplangError> {
    let Conn::Mongo(mongo) = db.current() else {
        return Err(data_error("Xiriirka MongoDB lama helin."));
    };
    let Some(session) = mongo.session.as_mut() else {
        return Ok(());
    };
    let result = if commit {
        session.commit_transaction().run()
    } else {
        session.abort_transaction().run()
    };
    mongo.session = None;
    result.map_err(|error| data_error(error.to_string()))
}

fn exec(db: &mut Database, sql: &str) -> Result<u64, SoplangError> {
    match db.current() {
        Conn::Sqlite(conn) => conn
            .execute(sql, [])
            .map(|count| count as u64)
            .map_err(|error| data_error(error.to_string())),
        Conn::Postgres(client) => client
            .execute(sql, &[])
            .map_err(|error| data_error(error.to_string())),
        Conn::Mysql(conn) => {
            conn.query_drop(sql)
                .map_err(|error| data_error(error.to_string()))?;
            Ok(conn.affected_rows())
        }
        Conn::SqlServer(server) => server.runtime.block_on(async {
            server
                .client
                .execute(sql, &[])
                .await
                .map(|result| result.total())
                .map_err(|error| data_error(error.to_string()))
        }),
        Conn::Mongo(_) => Err(data_error("SQL lama isticmaalo MongoDB.")),
    }
}

struct SqlRow {
    values: Vec<(String, SqlValue)>,
}

impl SqlRow {
    fn get(&self, index: usize) -> Option<&SqlValue> {
        self.values.get(index).map(|(_, value)| value)
    }
}

fn query(db: &mut Database, sql: &str) -> Result<Vec<SqlRow>, SoplangError> {
    match db.current() {
        Conn::Sqlite(conn) => {
            let mut statement = conn
                .prepare(sql)
                .map_err(|error| data_error(error.to_string()))?;
            let columns: Vec<String> = statement
                .column_names()
                .iter()
                .map(|name| (*name).to_string())
                .collect();
            let mut rows = statement
                .query([])
                .map_err(|error| data_error(error.to_string()))?;
            let mut out = Vec::new();
            while let Some(row) = rows.next().map_err(|error| data_error(error.to_string()))? {
                let mut values = Vec::new();
                for (index, name) in columns.iter().enumerate() {
                    values.push((name.clone(), sqlite_value(row, index)?));
                }
                out.push(SqlRow { values });
            }
            Ok(out)
        }
        Conn::Postgres(client) => {
            let rows = client
                .query(sql, &[])
                .map_err(|error| data_error(error.to_string()))?;
            let mut out = Vec::new();
            for row in rows {
                let mut values = Vec::new();
                for (index, column) in row.columns().iter().enumerate() {
                    values.push((column.name().to_string(), postgres_value(&row, index)?));
                }
                out.push(SqlRow { values });
            }
            Ok(out)
        }
        Conn::Mysql(conn) => {
            let result = conn
                .query_iter(sql)
                .map_err(|error| data_error(error.to_string()))?;
            let columns = result
                .columns()
                .as_ref()
                .iter()
                .map(|column| column.name_str().to_string())
                .collect::<Vec<_>>();
            let mut out = Vec::new();
            for row in result {
                let row = row.map_err(|error| data_error(error.to_string()))?;
                let mut values = Vec::new();
                for (index, name) in columns.iter().enumerate() {
                    values.push((name.clone(), mysql_value(row.as_ref(index).cloned())));
                }
                out.push(SqlRow { values });
            }
            Ok(out)
        }
        Conn::SqlServer(server) => server.runtime.block_on(async {
            let rows = server
                .client
                .simple_query(sql)
                .await
                .map_err(|error| data_error(error.to_string()))?
                .into_first_result()
                .await
                .map_err(|error| data_error(error.to_string()))?;
            let mut out = Vec::new();
            for row in rows {
                let mut values = Vec::new();
                for (column, cell) in row.cells() {
                    values.push((column.name().to_string(), mssql_value(cell)));
                }
                out.push(SqlRow { values });
            }
            Ok(out)
        }),
        Conn::Mongo(_) => Err(data_error("SQL lama isticmaalo MongoDB.")),
    }
}

fn sqlite_value(row: &rusqlite::Row<'_>, index: usize) -> Result<SqlValue, SoplangError> {
    let value: rusqlite::types::Value = row
        .get(index)
        .map_err(|error| data_error(error.to_string()))?;
    Ok(match value {
        rusqlite::types::Value::Null => SqlValue::Null,
        rusqlite::types::Value::Integer(value) => SqlValue::Int(value),
        rusqlite::types::Value::Real(value) => SqlValue::Float(value),
        rusqlite::types::Value::Text(value) => SqlValue::Text(value),
        rusqlite::types::Value::Blob(value) => {
            SqlValue::Text(String::from_utf8_lossy(&value).into())
        }
    })
}

fn postgres_value(row: &postgres::Row, index: usize) -> Result<SqlValue, SoplangError> {
    if let Ok(value) = row.try_get::<_, Option<i64>>(index) {
        return Ok(value.map(SqlValue::Int).unwrap_or(SqlValue::Null));
    }
    if let Ok(value) = row.try_get::<_, Option<f64>>(index) {
        return Ok(value.map(SqlValue::Float).unwrap_or(SqlValue::Null));
    }
    if let Ok(value) = row.try_get::<_, Option<bool>>(index) {
        return Ok(value.map(SqlValue::Bool).unwrap_or(SqlValue::Null));
    }
    if let Ok(value) = row.try_get::<_, Option<String>>(index) {
        return Ok(value.map(SqlValue::Text).unwrap_or(SqlValue::Null));
    }
    Ok(SqlValue::Null)
}

fn mysql_value(value: Option<mysql::Value>) -> SqlValue {
    match value {
        None | Some(mysql::Value::NULL) => SqlValue::Null,
        Some(mysql::Value::Int(value)) => SqlValue::Int(value),
        Some(mysql::Value::UInt(value)) => SqlValue::Int(value as i64),
        Some(mysql::Value::Float(value)) => SqlValue::Float(value as f64),
        Some(mysql::Value::Double(value)) => SqlValue::Float(value),
        Some(mysql::Value::Bytes(value)) => SqlValue::Text(String::from_utf8_lossy(&value).into()),
        Some(other) => SqlValue::Text(format!("{other:?}")),
    }
}

fn mssql_value(value: &tiberius::ColumnData<'_>) -> SqlValue {
    match value {
        tiberius::ColumnData::I32(Some(value)) => SqlValue::Int(i64::from(*value)),
        tiberius::ColumnData::I64(Some(value)) => SqlValue::Int(*value),
        tiberius::ColumnData::F64(Some(value)) => SqlValue::Float(*value),
        tiberius::ColumnData::Bit(Some(value)) => SqlValue::Bool(*value),
        tiberius::ColumnData::String(Some(value)) => SqlValue::Text(value.to_string()),
        _ => SqlValue::Null,
    }
}

fn mongo_insert(
    mongo: &mut MongoHandle,
    table: &str,
    fields: &HashMap<String, SqlValue>,
) -> Result<Value, SoplangError> {
    let collection = mongo.database.clone().collection::<Document>(table);
    let document = document(fields);
    let result = if let Some(session) = mongo.session.as_mut() {
        collection.insert_one(document).session(session).run()
    } else {
        collection.insert_one(document).run()
    };
    result.map_err(|error| data_error(error.to_string()))?;
    Ok(Value::Int(1))
}

fn mongo_find(
    mongo: &mut MongoHandle,
    table: &str,
    filter: &HashMap<String, SqlValue>,
) -> Result<Value, SoplangError> {
    let collection = mongo.database.clone().collection::<Document>(table);
    let filter = document(filter);
    let mut rows = Vec::new();
    if mongo.session.is_some() {
        let mut cursor = {
            let session = mongo.session.as_mut().unwrap();
            collection
                .find(filter)
                .session(session)
                .run()
                .map_err(|error| data_error(error.to_string()))?
        };
        let session = mongo.session.as_mut().unwrap();
        for document in cursor.iter(session).take(1000) {
            rows.push(document_to_value(
                document.map_err(|error| data_error(error.to_string()))?,
            ));
        }
    } else {
        let cursor = collection
            .find(filter)
            .run()
            .map_err(|error| data_error(error.to_string()))?;
        for document in cursor.take(1000) {
            rows.push(document_to_value(
                document.map_err(|error| data_error(error.to_string()))?,
            ));
        }
    }
    Ok(Value::List(std::rc::Rc::new(std::cell::RefCell::new(rows))))
}

fn mongo_update(
    mongo: &mut MongoHandle,
    table: &str,
    filter: &HashMap<String, SqlValue>,
    changes: &HashMap<String, SqlValue>,
) -> Result<Value, SoplangError> {
    let collection = mongo.database.clone().collection::<Document>(table);
    let filter = document(filter);
    let changes = doc! { "$set": document(changes) };
    let result = if let Some(session) = mongo.session.as_mut() {
        collection
            .update_many(filter, changes)
            .session(session)
            .run()
    } else {
        collection.update_many(filter, changes).run()
    }
    .map_err(|error| data_error(error.to_string()))?;
    Ok(Value::Int(result.modified_count as i64))
}

fn mongo_delete(
    mongo: &mut MongoHandle,
    table: &str,
    filter: &HashMap<String, SqlValue>,
) -> Result<Value, SoplangError> {
    let collection = mongo.database.clone().collection::<Document>(table);
    let filter = document(filter);
    let result = if let Some(session) = mongo.session.as_mut() {
        collection.delete_many(filter).session(session).run()
    } else {
        collection.delete_many(filter).run()
    }
    .map_err(|error| data_error(error.to_string()))?;
    Ok(Value::Int(result.deleted_count as i64))
}

fn document(fields: &HashMap<String, SqlValue>) -> Document {
    fields
        .iter()
        .map(|(name, value)| (name.clone(), bson_value(value)))
        .collect()
}

fn bson_value(value: &SqlValue) -> Bson {
    match value {
        SqlValue::Null => Bson::Null,
        SqlValue::Int(value) => Bson::Int64(*value),
        SqlValue::Float(value) => Bson::Double(*value),
        SqlValue::Text(value) => Bson::String(value.clone()),
        SqlValue::Bool(value) => Bson::Boolean(*value),
    }
}

fn document_to_value(document: Document) -> Value {
    let mut fields = HashMap::new();
    for (name, value) in document {
        if name == "_id" {
            continue;
        }
        fields.insert(
            name,
            match value {
                Bson::Null => Value::Null,
                Bson::Int32(value) => Value::Int(i64::from(value)),
                Bson::Int64(value) => Value::Int(value),
                Bson::Double(value) => Value::Float(value),
                Bson::String(value) => Value::Str(value),
                Bson::Boolean(value) => Value::Bool(value),
                other => Value::Str(other.to_string()),
            },
        );
    }
    object_from(fields)
}

fn require_table(db: &Database, name: &str) -> Result<schema::Table, SoplangError> {
    if !schema::ident(name) {
        return Err(data_error(format!("Magaca jadwalka '{name}' ma saxna.")));
    }
    db.schema
        .as_ref()
        .and_then(|schema| schema.table(name).cloned())
        .ok_or_else(|| data_error(format!("Jadwalka '{name}' kuma jiro schema.")))
}

fn reject_unknown(
    table: &schema::Table,
    fields: &HashMap<String, SqlValue>,
) -> Result<(), SoplangError> {
    for name in fields.keys() {
        if !table.columns.iter().any(|column| &column.name == name) {
            return Err(data_error(format!(
                "Tiirka '{name}' kuma jiro jadwalka '{}'.",
                table.name
            )));
        }
    }
    Ok(())
}

fn object_fields(value: &Value) -> Result<HashMap<String, SqlValue>, SoplangError> {
    let Value::Object(object) = value else {
        return Err(data_error("Xogta jadwalka waa inay noqotaa walax."));
    };
    let object = object
        .try_borrow()
        .map_err(|_| data_error("Walaxda lama akhriyi karo."))?;
    let mut fields = HashMap::new();
    for (name, value) in object.iter() {
        if !schema::ident(name) {
            return Err(data_error(format!("Magaca tiirka '{name}' ma saxna.")));
        }
        fields.insert(name.clone(), sql_value(value)?);
    }
    Ok(fields)
}

fn sql_value(value: &Value) -> Result<SqlValue, SoplangError> {
    match value {
        Value::Null => Ok(SqlValue::Null),
        Value::Int(value) => Ok(SqlValue::Int(*value)),
        Value::Float(value) if value.is_finite() => Ok(SqlValue::Float(*value)),
        Value::Bool(value) => Ok(SqlValue::Bool(*value)),
        Value::Str(value) => Ok(SqlValue::Text(value.clone())),
        _ => Err(data_error(
            "Qiimaha tiirka waa inuu noqdaa abn, jajab, bool, qoraal, ama maran.",
        )),
    }
}

fn rows_to_value(rows: &[SqlRow]) -> Value {
    Value::List(std::rc::Rc::new(std::cell::RefCell::new(
        rows.iter()
            .map(|row| {
                let mut fields = HashMap::new();
                for (name, value) in &row.values {
                    fields.insert(
                        name.clone(),
                        match value {
                            SqlValue::Null => Value::Null,
                            SqlValue::Int(value) => Value::Int(*value),
                            SqlValue::Float(value) => Value::Float(*value),
                            SqlValue::Text(value) => Value::Str(value.clone()),
                            SqlValue::Bool(value) => Value::Bool(*value),
                        },
                    );
                }
                object_from(fields)
            })
            .collect(),
    )))
}

fn with_db<T>(
    id: i64,
    body: impl FnOnce(&mut Database) -> Result<T, SoplangError>,
) -> Result<T, SoplangError> {
    let mut databases = DATABASES
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let db = databases
        .get_or_insert_with(HashMap::new)
        .get_mut(&id)
        .ok_or_else(|| data_error("Xogta waa la xidhay."))?;
    body(db)
}

fn handle(id: i64, kind: &str) -> Value {
    object(&[
        ("aqoonsi", Value::Int(id)),
        ("nooc", Value::Str(kind.into())),
    ])
}

fn object(fields: &[(&str, Value)]) -> Value {
    object_from(
        fields
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect(),
    )
}

fn object_from(fields: HashMap<String, Value>) -> Value {
    Value::Object(std::rc::Rc::new(std::cell::RefCell::new(fields)))
}

fn data_error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E059_DATA),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> schema::Table {
        parse_schema("jadwal qof {\n  id abn furaha\n  magac qoraal\n}\n")
            .unwrap()
            .tables
            .remove(0)
    }

    #[test]
    fn each_sql_dialect_has_its_own_key_column() {
        let table = sample();
        let sqlite = create_table(DriverKind::Sqlite, &table);
        let postgres = create_table(DriverKind::Postgres, &table);
        let mysql = create_table(DriverKind::Mysql, &table);
        let server = create_table(DriverKind::SqlServer, &table);
        assert!(sqlite.contains("AUTOINCREMENT"));
        assert!(postgres.contains("GENERATED BY DEFAULT AS IDENTITY"));
        assert_eq!(postgres, create_table(DriverKind::Cockroach, &table));
        assert!(mysql.contains("AUTO_INCREMENT"));
        assert_eq!(mysql, create_table(DriverKind::Mariadb, &table));
        assert!(server.contains("IDENTITY(1,1)"));
        assert_eq!(
            literal(DriverKind::Sqlite, &SqlValue::Text("a'b".into())),
            "'a''b'"
        );
    }
}
