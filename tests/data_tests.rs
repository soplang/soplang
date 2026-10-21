//! SQLite exercises the ORM. The other six connectors are real clients with
//! dialect tests here and live tests behind `SOPLANG_*` when a server exists.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use soplang::data::{self, capabilities};
use soplang::Value;

const SCHEMA: &str = "jadwal qof {\n  id abn furaha\n  magac qoraal\n  da abn?\n}\n";

fn temp_db(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "soplang_{name}_{}_{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = fs::remove_file(&path);
    path
}

fn obj(fields: &[(&str, Value)]) -> Value {
    Value::Object(Rc::new(RefCell::new(
        fields
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect::<HashMap<_, _>>(),
    )))
}

fn id_of(value: &Value) -> i64 {
    let Value::Object(object) = value else {
        panic!("handle");
    };
    let Value::Int(id) = object.borrow().get("aqoonsi").cloned().unwrap() else {
        panic!("id");
    };
    id
}

fn names(value: &Value) -> Vec<String> {
    let Value::List(items) = value else {
        panic!("list");
    };
    items
        .borrow()
        .iter()
        .map(|row| {
            let Value::Object(object) = row else {
                panic!("row");
            };
            match object.borrow().get("magac") {
                Some(Value::Str(text)) => text.clone(),
                other => panic!("magac {other:?}"),
            }
        })
        .collect()
}

#[test]
fn sqlite_migrates_queries_and_rolls_back() {
    let path = temp_db("orm");
    let handle = data::open("sqlite", path.to_str().unwrap(), 2).unwrap();
    let id = id_of(&handle);
    data::load_schema(id, SCHEMA).unwrap();
    data::migrate(id).unwrap();
    let failed = data::transaction::<()>(id, || {
        data::insert(id, "qof", &obj(&[("magac", Value::Str("Amina".into()))]))?;
        Err(soplang::error::runtime_error("jooji", 0, 0))
    });
    assert!(failed.is_err());
    assert!(names(&data::find(id, "qof", None).unwrap()).is_empty());

    data::transaction::<()>(id, || {
        data::insert(id, "qof", &obj(&[("magac", Value::Str("Amina".into()))]))?;
        Ok(())
    })
    .unwrap();
    assert_eq!(names(&data::find(id, "qof", None).unwrap()), ["Amina"]);
    data::update(
        id,
        "qof",
        &obj(&[("magac", Value::Str("Amina".into()))]),
        &obj(&[("magac", Value::Str("Zahra".into()))]),
    )
    .unwrap();
    assert_eq!(names(&data::find(id, "qof", None).unwrap()), ["Zahra"]);
    data::delete(id, "qof", &obj(&[("magac", Value::Str("Zahra".into()))])).unwrap();
    assert!(names(&data::find(id, "qof", None).unwrap()).is_empty());
    assert!(data::insert(id, "qof", &obj(&[("sir", Value::Str("x".into()))])).is_err());
}

#[test]
fn every_connector_reports_a_capability() {
    for kind in [
        "sqlite",
        "postgres",
        "cockroach",
        "mysql",
        "mariadb",
        "sqlserver",
        "mongo",
    ] {
        let value = capabilities(kind).unwrap();
        let Value::Object(object) = value else {
            panic!("object");
        };
        assert_eq!(object.borrow().get("nooc"), Some(&Value::Str(kind.into())));
        assert_eq!(object.borrow().get("geli"), Some(&Value::Bool(true)));
        assert_eq!(object.borrow().get("macaamil"), Some(&Value::Bool(true)));
    }
    assert!(capabilities("fake").is_err());
}

#[test]
fn bundled_xog_reads_sqlite() {
    let path = temp_db("xog");
    let file = path.with_extension("sop");
    let schema = path.with_extension("schema");
    fs::write(&schema, SCHEMA).unwrap();
    let source = format!(
        "keen \"xog\" sida xog\ndoor db = xog.fur(\"sqlite\", \"{}\")\ndoor qoraalka = akhrifayl(\"{}\")\nxog.schema(db, qoraalka)\nxog.guuri(db)\nxog.ku_geli(db, \"qof\", {{ magac: \"Amina\" }})\ndoor rows = xog.raadi(db, \"qof\")\nhubi(dherer(rows), 1)\nhubi(rows[0].magac, \"Amina\")\n",
        path.display().to_string().replace('\\', "/"),
        schema.display().to_string().replace('\\', "/")
    );
    fs::write(&file, &source).unwrap();
    soplang::run_source(&source, Some(&file), false, false, false).unwrap();
}

fn live(kind: &str, target: &str) {
    let handle = data::open(kind, target, 1).unwrap();
    let id = id_of(&handle);
    data::load_schema(id, SCHEMA).unwrap();
    data::migrate(id).unwrap();
    let marker = format!("live-{kind}-{}", std::process::id());
    data::insert(id, "qof", &obj(&[("magac", Value::Str(marker.clone()))])).unwrap();
    let found = data::find(
        id,
        "qof",
        Some(&obj(&[("magac", Value::Str(marker.clone()))])),
    )
    .unwrap();
    assert_eq!(names(&found), [marker.clone()]);
    data::delete(id, "qof", &obj(&[("magac", Value::Str(marker))])).unwrap();
}

#[test]
#[ignore]
fn live_postgres() {
    live("postgres", &std::env::var("SOPLANG_POSTGRES").unwrap());
}

#[test]
#[ignore]
fn live_cockroach() {
    live("cockroach", &std::env::var("SOPLANG_COCKROACH").unwrap());
}

#[test]
#[ignore]
fn live_mysql() {
    live("mysql", &std::env::var("SOPLANG_MYSQL").unwrap());
}

#[test]
#[ignore]
fn live_mariadb() {
    live("mariadb", &std::env::var("SOPLANG_MARIADB").unwrap());
}

#[test]
#[ignore]
fn live_sqlserver() {
    live("sqlserver", &std::env::var("SOPLANG_SQLSERVER").unwrap());
}

#[test]
#[ignore]
fn live_mongo() {
    live("mongo", &std::env::var("SOPLANG_MONGO").unwrap());
}
