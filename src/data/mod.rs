//! Schema, migrations, pools, and real database connectors.
//!
//! SQLite is embedded. PostgreSQL and CockroachDB share the Postgres wire
//! protocol. MySQL and MariaDB share the MySQL protocol. MongoDB and SQL Server
//! use their own clients. Each connector executes statements through that
//! client; none of them return canned rows.

mod schema;
mod generate;
mod migration;
mod mongo;
mod sql;

use crate::runtime::with_databases;
use sql::{Param, Statement};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use mongodb::bson::{doc, Bson, Document};
use mongodb::sync::Client as MongoClient;
use mysql::prelude::Queryable;
use postgres::NoTls;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

pub use schema::{format_schema, ident, parse_schema, parse_schema_file, ColumnType, Schema};
pub use generate::{generate_client, GENERATED_HEADER};

use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};
use crate::runtime::value::Value;

static NEXT_ID: AtomicI64 = AtomicI64::new(1);

pub(crate) struct Database {
    kind: DriverKind,
    schema: Option<Schema>,
    conns: Vec<Conn>,
    cursor: usize,
    pinned: Option<usize>,
}

impl Database {
    fn current(&mut self) -> &mut Conn {
        let index = self.pinned.unwrap_or(self.cursor);
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
    if !(1..=8).contains(&pool) {
        return Err(data_error(
            "Tirada xiriirradu waa inay u dhexaysaa 1 iyo 8.",
        ));
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let shared_memory = format!(
        "file:soplang-memory-{}-{id}?mode=memory&cache=shared",
        std::process::id()
    );
    let target = if matches!(kind, DriverKind::Sqlite) && target == ":memory:" {
        shared_memory.as_str()
    } else {
        target
    };
    let mut conns = Vec::with_capacity(pool);
    for _ in 0..pool {
        conns.push(connect(kind, target)?);
    }
    with_databases(|databases| {
        databases.borrow_mut().insert(
            id,
            Database {
                kind,
                schema: None,
                conns,
                cursor: 0,
                pinned: None,
            },
        );
    });
    Ok(handle(id, kind.name()))
}

/// Close a database owned by the currently active execution context.
pub fn close(id: i64) -> Result<Value, SoplangError> {
    with_databases(|databases| {
        let mut databases = databases
            .try_borrow_mut()
            .map_err(|_| data_error("Xogta hadda waa la isticmaalayaa."))?;
        let db = databases
            .get(&id)
            .ok_or_else(|| data_error("Xogta waa la xidhay."))?;
        if db.pinned.is_some() {
            return Err(data_error(
                "Xogta lama xidhi karo inta macaamil furan yahay.",
            ));
        }
        databases.remove(&id);
        Ok(Value::Null)
    })
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
        if db.pinned.is_some() {
            return Err(data_error(
                "Guurinta lama samayn karo inta macaamil furan yahay.",
            ));
        }
        let atomic = !matches!(
            db.kind,
            DriverKind::Mysql | DriverKind::Mariadb | DriverKind::Mongo
        );
        if atomic {
            begin(db)?;
        }
        let result = apply_migration(db, &schema);
        if atomic {
            let finish = if result.is_ok() {
                commit(db)
            } else {
                rollback(db)
            };
            if finish.is_err() && result.is_ok() {
                let _ = rollback(db);
            }
            db.pinned = None;
            finish?;
        }
        result?;
        Ok(Value::Null)
    })
}

pub fn insert(id: i64, table: &str, value: &Value) -> Result<Value, SoplangError> {
    with_db(id, |db| {
        let table = require_table(db, table)?;
        let mut fields = object_fields(value)?;
        validate_fields(&table, &mut fields, true)?;
        match db.current() {
            Conn::Mongo(mongo) => mongo_insert(mongo, &table, &fields),
            _ => {
                let sql = sql::insert(db.kind, &table, &fields);
                let count = exec_statement(db, &sql)?;
                Ok(Value::Int(count as i64))
            }
        }
    })
}

pub fn find(id: i64, table: &str, filter: Option<&Value>) -> Result<Value, SoplangError> {
    with_db(id, |db| {
        let table = require_table(db, table)?;
        let mut filter = match filter {
            Some(value) => object_fields(value)?,
            None => HashMap::new(),
        };
        validate_fields(&table, &mut filter, false)?;
        match db.current() {
            Conn::Mongo(mongo) => mongo_find(mongo, &table, &filter),
            _ => {
                let sql = sql::select(db.kind, &table, &filter);
                let mut rows = query_statement(db, &sql)?;
                for row in &mut rows {
                    for (name, value) in &mut row.values {
                        let column = table
                            .columns
                            .iter()
                            .find(|column| &column.name == name)
                            .ok_or_else(|| data_error("Jadwalku ma waafaqsana schema."))?;
                        match (&*value, column.ty) {
                            (SqlValue::Int(0), ColumnType::Bool) => *value = SqlValue::Bool(false),
                            (SqlValue::Int(1), ColumnType::Bool) => *value = SqlValue::Bool(true),
                            (SqlValue::Int(number), ColumnType::Float) => {
                                *value = SqlValue::Float(*number as f64)
                            }
                            _ => {}
                        }
                        validate_fields(
                            &table,
                            &mut HashMap::from([(name.clone(), value.clone())]),
                            false,
                        )?;
                    }
                }
                Ok(rows_to_value(&rows))
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
        let mut filter = object_fields(filter)?;
        let mut changes = object_fields(changes)?;
        validate_fields(&table, &mut filter, false)?;
        validate_fields(&table, &mut changes, false)?;
        if changes.is_empty() {
            return Err(data_error(
                "Beddelku waa inuu lahaadaa ugu yaraan hal tiir.",
            ));
        }
        match db.current() {
            Conn::Mongo(mongo) => mongo_update(mongo, &table.name, &filter, &changes),
            _ => Ok(Value::Int(
                exec_statement(db, &sql::update(db.kind, &table, &filter, &changes))? as i64,
            )),
        }
    })
}

pub fn delete(id: i64, table: &str, filter: &Value) -> Result<Value, SoplangError> {
    with_db(id, |db| {
        let table = require_table(db, table)?;
        let mut filter = object_fields(filter)?;
        validate_fields(&table, &mut filter, false)?;
        match db.current() {
            Conn::Mongo(mongo) => mongo_delete(mongo, &table.name, &filter),
            _ => Ok(Value::Int(
                exec_statement(db, &sql::delete(db.kind, &table, &filter))? as i64,
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
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
    let finished = with_db(id, |db| {
        let outcome = match &result {
            Ok(Ok(_)) => {
                let result = commit(db);
                if result.is_err() {
                    let _ = rollback(db);
                }
                result
            }
            Ok(Err(_)) | Err(_) => rollback(db),
        };
        db.pinned = None;
        outcome
    });
    match result {
        Ok(result) => {
            finished?;
            result
        }
        Err(panic) => std::panic::resume_unwind(panic),
    }
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
            let conn = rusqlite::Connection::open(target).map_err(|error| driver_error(error))?;
            conn.busy_timeout(Duration::from_secs(5))
                .map_err(|error| driver_error(error))?;
            conn.execute_batch("PRAGMA foreign_keys = ON;")
                .map_err(|error| driver_error(error))?;
            Ok(Conn::Sqlite(conn))
        }
        DriverKind::Postgres | DriverKind::Cockroach => {
            let client =
                postgres::Client::connect(target, NoTls).map_err(|error| driver_error(error))?;
            Ok(Conn::Postgres(client))
        }
        DriverKind::Mysql | DriverKind::Mariadb => {
            let conn = mysql::Conn::new(target).map_err(|error| driver_error(error))?;
            Ok(Conn::Mysql(conn))
        }
        DriverKind::Mongo => {
            let client = MongoClient::with_uri_str(target).map_err(|error| driver_error(error))?;
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
        .map_err(|error| driver_error(error))?;
    let client = runtime.block_on(async {
        let config =
            tiberius::Config::from_ado_string(target).map_err(|error| driver_error(error))?;
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(|error| driver_error(error))?;
        tcp.set_nodelay(true).map_err(|error| driver_error(error))?;
        tiberius::Client::connect(config, tcp.compat_write())
            .await
            .map_err(|error| driver_error(error))
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

fn mongo_validator(table: &schema::Table) -> Document {
    let mut properties = doc! { "_id": {} };
    let mut required = Vec::new();
    for column in &table.columns {
        let mut types = match column.ty {
            ColumnType::Int => vec!["int", "long"],
            ColumnType::Text => vec!["string"],
            ColumnType::Float => vec!["int", "long", "double"],
            ColumnType::Bool => vec!["bool"],
        };
        if column.optional {
            types.push("null");
        }
        properties.insert(&column.name, doc! { "bsonType": types });
        if !column.optional {
            required.push(column.name.clone());
        }
    }
    doc! { "$jsonSchema": { "bsonType": "object", "required": required, "properties": properties, "additionalProperties": false } }
}

fn apply_migration(db: &mut Database, schema: &Schema) -> Result<(), SoplangError> {
    migration::preflight(db, schema)?;
    if matches!(db.kind, DriverKind::Mongo) {
        let Conn::Mongo(mongo) = db.current() else {
            unreachable!()
        };
        let existing = mongo
            .database
            .list_collection_names()
            .run()
            .map_err(|error| driver_error(error))?;
        for table in &schema.tables {
            let validator = mongo_validator(table);
            let command = if existing.iter().any(|name| name == &table.name) {
                doc! { "collMod": &table.name, "validator": validator, "validationLevel": "strict" }
            } else {
                doc! { "create": &table.name, "validator": validator, "validationLevel": "strict" }
            };
            mongo
                .database
                .run_command(command)
                .run()
                .map_err(|error| driver_error(error))?;
            let key = table.columns.iter().find(|column| column.key).unwrap();
            // MongoDB already creates the unique primary _id index.
            if key.name == "_id" {
                continue;
            }
            let index = mongodb::IndexModel::builder()
                .keys(doc! { &key.name: 1 })
                .options(
                    mongodb::options::IndexOptions::builder()
                        .unique(true)
                        .build(),
                )
                .build();
            mongo
                .database
                .collection::<Document>(&table.name)
                .create_index(index)
                .run()
                .map_err(|error| driver_error(error))?;
        }
        return migration::record(db, schema);
    }
    for table in &schema.tables {
        exec(db, &create_table(db.kind, table))?;
        let existing = existing_columns(db, &table.name)?;
        for column in &table.columns {
            if !existing.iter().any(|name| name == &column.name) && !column.key {
                exec(db, &add_column(db.kind, &table.name, column))?;
            }
        }
    }
    migration::record(db, schema)
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
            table.name,
            sql::identifier(kind, &table.name)
        ),
        _ => format!(
            "CREATE TABLE IF NOT EXISTS {} ({columns});",
            sql::identifier(kind, &table.name)
        ),
    }
}

fn column_sql(kind: DriverKind, column: &schema::Column) -> String {
    let name = sql::identifier(kind, &column.name);
    if column.key {
        return match kind {
            DriverKind::Sqlite => format!("{name} INTEGER PRIMARY KEY AUTOINCREMENT"),
            DriverKind::Postgres | DriverKind::Cockroach => format!(
                "{} BIGINT GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY",
                name
            ),
            DriverKind::Mysql | DriverKind::Mariadb => {
                format!("{name} BIGINT PRIMARY KEY AUTO_INCREMENT")
            }
            DriverKind::SqlServer => format!("{name} BIGINT IDENTITY(1,1) PRIMARY KEY"),
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
    format!(
        "{name} {ty}{}",
        if column.optional { "" } else { " NOT NULL" }
    )
}

fn add_column(kind: DriverKind, table: &str, column: &schema::Column) -> String {
    let table = sql::identifier(kind, table);
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
        DriverKind::Sqlite => format!("PRAGMA table_info({})", sql::identifier(db.kind, table)),
        DriverKind::SqlServer => format!(
            "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = '{table}'"
        ),
        DriverKind::Mysql | DriverKind::Mariadb => format!(
            "SELECT column_name FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = '{table}'"
        ),
        _ => format!(
            "SELECT column_name FROM information_schema.columns WHERE table_schema = current_schema() AND table_name = '{table}'"
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
    exec(db, "ROLLBACK")?;
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
        .map_err(|error| driver_error(error))?;
    session
        .start_transaction()
        .run()
        .map_err(|error| driver_error(error))?;
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
    result.map_err(|error| driver_error(error))
}

fn exec(db: &mut Database, sql: &str) -> Result<u64, SoplangError> {
    // Transaction controls and DDL use the driver's batch protocol. MySQL
    // cannot prepare every control command, and a SQL Server RPC must not
    // change the surrounding session's transaction count.
    match db.current() {
        Conn::Mysql(conn) => {
            conn.query_drop(sql).map_err(driver_error)?;
            return Ok(conn.affected_rows());
        }
        Conn::SqlServer(server) => {
            return server.runtime.block_on(async {
                server
                    .client
                    .simple_query(sql)
                    .await
                    .map_err(driver_error)?
                    .into_results()
                    .await
                    .map_err(driver_error)?;
                Ok(0)
            })
        }
        _ => {}
    }
    exec_statement(
        db,
        &Statement {
            text: sql.into(),
            params: Vec::new(),
        },
    )
}

fn exec_statement(db: &mut Database, statement: &Statement) -> Result<u64, SoplangError> {
    let sql = &statement.text;
    let params = &statement.params;
    match db.current() {
        Conn::Sqlite(conn) => conn
            .execute(
                sql,
                rusqlite::params_from_iter(params.iter().map(Param::sqlite)),
            )
            .map(|count| count as u64)
            .map_err(|error| driver_error(error)),
        Conn::Postgres(client) => {
            let values = params.iter().map(Param::postgres).collect::<Vec<_>>();
            let refs = values
                .iter()
                .map(|v| v.as_ref() as &(dyn postgres::types::ToSql + Sync))
                .collect::<Vec<_>>();
            client
                .execute(sql.as_str(), &refs)
                .map_err(|error| driver_error(error))
        }
        Conn::Mysql(conn) => {
            conn.exec_drop(
                sql,
                mysql::Params::Positional(params.iter().map(Param::mysql).collect()),
            )
            .map_err(|error| driver_error(error))?;
            Ok(conn.affected_rows())
        }
        Conn::SqlServer(server) => server.runtime.block_on(async {
            let values = params.iter().map(Param::sqlserver).collect::<Vec<_>>();
            let refs = values
                .iter()
                .map(|v| v.as_ref() as &dyn tiberius::ToSql)
                .collect::<Vec<_>>();
            server
                .client
                .execute(sql, &refs)
                .await
                .map(|result| result.total())
                .map_err(|error| driver_error(error))
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
    query_statement(
        db,
        &Statement {
            text: sql.into(),
            params: Vec::new(),
        },
    )
}

fn query_statement(db: &mut Database, statement: &Statement) -> Result<Vec<SqlRow>, SoplangError> {
    let sql = &statement.text;
    let params = &statement.params;
    match db.current() {
        Conn::Sqlite(conn) => {
            let mut statement = conn.prepare(sql).map_err(|error| driver_error(error))?;
            let columns: Vec<String> = statement
                .column_names()
                .iter()
                .map(|name| (*name).to_string())
                .collect();
            let mut rows = statement
                .query(rusqlite::params_from_iter(params.iter().map(Param::sqlite)))
                .map_err(|error| driver_error(error))?;
            let mut out = Vec::new();
            while let Some(row) = rows.next().map_err(|error| driver_error(error))? {
                let mut values = Vec::new();
                for (index, name) in columns.iter().enumerate() {
                    values.push((name.clone(), sqlite_value(row, index)?));
                }
                out.push(SqlRow { values });
            }
            Ok(out)
        }
        Conn::Postgres(client) => {
            let values = params.iter().map(Param::postgres).collect::<Vec<_>>();
            let refs = values
                .iter()
                .map(|v| v.as_ref() as &(dyn postgres::types::ToSql + Sync))
                .collect::<Vec<_>>();
            let rows = client
                .query(sql.as_str(), &refs)
                .map_err(|error| driver_error(error))?;
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
                .exec_iter(
                    sql,
                    mysql::Params::Positional(params.iter().map(Param::mysql).collect()),
                )
                .map_err(|error| driver_error(error))?;
            let columns = result
                .columns()
                .as_ref()
                .iter()
                .map(|column| column.name_str().to_string())
                .collect::<Vec<_>>();
            let mut out = Vec::new();
            for row in result {
                let row = row.map_err(|error| driver_error(error))?;
                let mut values = Vec::new();
                for (index, name) in columns.iter().enumerate() {
                    values.push((name.clone(), mysql_value(row.as_ref(index).cloned())));
                }
                out.push(SqlRow { values });
            }
            Ok(out)
        }
        Conn::SqlServer(server) => server.runtime.block_on(async {
            let values = params.iter().map(Param::sqlserver).collect::<Vec<_>>();
            let refs = values
                .iter()
                .map(|v| v.as_ref() as &dyn tiberius::ToSql)
                .collect::<Vec<_>>();
            let rows = server
                .client
                .query(sql, &refs)
                .await
                .map_err(|error| driver_error(error))?
                .into_first_result()
                .await
                .map_err(|error| driver_error(error))?;
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
    let value: rusqlite::types::Value = row.get(index).map_err(|error| driver_error(error))?;
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
    table: &schema::Table,
    fields: &HashMap<String, SqlValue>,
) -> Result<Value, SoplangError> {
    let collection = mongo.database.clone().collection::<Document>(&table.name);
    let mut document = document(fields);
    let key = table.columns.iter().find(|column| column.key).unwrap();
    let counters = mongo.database.collection::<Document>("soplang_counters");
    if !fields.contains_key(&key.name) {
        mongo::baseline_counter(mongo, table)?;
        let action = counters
            .find_one_and_update(
                doc! { "_id": &table.name },
                doc! { "$inc": { "value": 1_i64 } },
            )
            .upsert(true)
            .return_document(mongodb::options::ReturnDocument::After);
        let counter = if let Some(session) = mongo.session.as_mut() {
            action.session(session).run()
        } else {
            action.run()
        }
        .map_err(|error| driver_error(error))?
        .ok_or_else(|| data_error("Furaha cusub lama abuurin."))?;
        let id = counter
            .get_i64("value")
            .map_err(|error| driver_error(error))?;
        document.insert(&key.name, id);
    } else if let Some(SqlValue::Int(id)) = fields.get(&key.name) {
        let action = counters
            .update_one(
                doc! { "_id": &table.name },
                doc! { "$max": { "value": *id } },
            )
            .upsert(true);
        if let Some(session) = mongo.session.as_mut() {
            action.session(session).run()
        } else {
            action.run()
        }
        .map_err(|error| driver_error(error))?;
    }
    let result = if let Some(session) = mongo.session.as_mut() {
        collection.insert_one(document).session(session).run()
    } else {
        collection.insert_one(document).run()
    };
    result.map_err(|error| driver_error(error))?;
    Ok(Value::Int(1))
}

fn mongo_find(
    mongo: &mut MongoHandle,
    table: &schema::Table,
    filter: &HashMap<String, SqlValue>,
) -> Result<Value, SoplangError> {
    let collection = mongo.database.clone().collection::<Document>(&table.name);
    let filter = document(filter);
    let mut rows = Vec::new();
    if mongo.session.is_some() {
        let mut cursor = {
            let session = mongo.session.as_mut().unwrap();
            collection
                .find(filter)
                .projection(mongo::projection(table))
                .limit(1000)
                .session(session)
                .run()
                .map_err(|error| driver_error(error))?
        };
        let session = mongo.session.as_mut().unwrap();
        for document in cursor.iter(session).take(1000) {
            rows.push(mongo::decode_row(
                document.map_err(|error| driver_error(error))?,
                table,
            )?);
        }
    } else {
        let cursor = collection
            .find(filter)
            .projection(mongo::projection(table))
            .limit(1000)
            .run()
            .map_err(|error| driver_error(error))?;
        for document in cursor.take(1000) {
            rows.push(mongo::decode_row(
                document.map_err(|error| driver_error(error))?,
                table,
            )?);
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
    .map_err(|error| driver_error(error))?;
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
    .map_err(|error| driver_error(error))?;
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


fn require_table(db: &Database, name: &str) -> Result<schema::Table, SoplangError> {
    if !schema::ident(name) {
        return Err(data_error(format!("Magaca jadwalka '{name}' ma saxna.")));
    }
    db.schema
        .as_ref()
        .and_then(|schema| schema.table(name).cloned())
        .ok_or_else(|| data_error(format!("Jadwalka '{name}' kuma jiro schema.")))
}

fn validate_fields(
    table: &schema::Table,
    fields: &mut HashMap<String, SqlValue>,
    insert: bool,
) -> Result<(), SoplangError> {
    for (name, value) in fields.iter_mut() {
        let column = table
            .columns
            .iter()
            .find(|column| &column.name == name)
            .ok_or_else(|| {
                data_error(format!(
                    "Tiirka '{name}' kuma jiro jadwalka '{}'.",
                    table.name
                ))
            })?;
        let valid = match (&*value, column.ty) {
            (SqlValue::Null, _) => column.optional,
            (SqlValue::Int(_), ColumnType::Int)
            | (SqlValue::Float(_), ColumnType::Float)
            | (SqlValue::Text(_), ColumnType::Text)
            | (SqlValue::Bool(_), ColumnType::Bool) => true,
            (SqlValue::Int(number), ColumnType::Float) => {
                *value = SqlValue::Float(*number as f64);
                true
            }
            _ => false,
        };
        if !valid {
            return Err(data_error(format!(
                "Qiimaha tiirka '{name}' ma waafaqsana noociisa."
            )));
        }
    }
    if insert {
        for column in &table.columns {
            if !fields.contains_key(&column.name) && !column.key {
                if column.optional {
                    fields.insert(column.name.clone(), SqlValue::Null);
                } else {
                    return Err(data_error(format!(
                        "Tiirka '{}' waa loo baahan yahay.",
                        column.name
                    )));
                }
            }
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

fn runtime_value(value: &SqlValue) -> Value {
    match value {
        SqlValue::Null => Value::Null,
        SqlValue::Int(value) => Value::Int(*value),
        SqlValue::Float(value) => Value::Float(*value),
        SqlValue::Text(value) => Value::Str(value.clone()),
        SqlValue::Bool(value) => Value::Bool(*value),
    }
}

fn rows_to_value(rows: &[SqlRow]) -> Value {
    Value::List(std::rc::Rc::new(std::cell::RefCell::new(
        rows.iter()
            .map(|row| {
                let mut fields = HashMap::new();
                for (name, value) in &row.values {
                    fields.insert(name.clone(), runtime_value(value));
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
    with_databases(|databases| {
        let mut databases = databases
            .try_borrow_mut()
            .map_err(|_| data_error("Xogta hadda waa la isticmaalayaa."))?;
        let db = databases
            .get_mut(&id)
            .ok_or_else(|| data_error("Xogta waa la xidhay."))?;
        let result = body(db);
        if db.pinned.is_none() {
            db.cursor = (db.cursor + 1) % db.conns.len();
        }
        result
    })
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

fn driver_error(error: impl std::fmt::Display) -> SoplangError {
    runtime_error_ex(
        "Hawlgalka kaydka xogtu ma hirgelin.",
        0,
        0,
        ErrorMeta::default()
            .with_code(codes::E059_DATA)
            .with_details(error.to_string()),
    )
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
    }
}
