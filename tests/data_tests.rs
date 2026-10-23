//! SQLite exercises the ORM. The other six connectors are real clients with
//! dialect tests here and live tests behind `SOPLANG_*` when a server exists.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

use soplang::data::{self, capabilities};
use soplang::Value;

const SCHEMA: &str =
    "jadwal qof {\n  id abn furaha\n  magac qoraal\n  da abn?\n  diyaar bool?\n  qiime jajab?\n}\n";

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
fn schemas_enforce_types_nullability_and_unique_columns() {
    assert!(
        data::parse_schema("jadwal t {\n id abn furaha\n name qoraal\n name abn\n}\n").is_err()
    );
    assert!(data::parse_schema("jadwal soplang_guuri {\n id abn furaha\n}\n").is_err());
    let context = soplang::runtime::RuntimeContext::new();
    context.with(|| {
        let id = id_of(&data::open("sqlite", ":memory:", 2).unwrap());
        data::load_schema(id, SCHEMA).unwrap();
        data::migrate(id).unwrap();
        for fields in [
            obj(&[]),
            obj(&[("magac", Value::Int(42))]),
            obj(&[("magac", Value::Null)]),
            obj(&[
                ("magac", Value::Str("ok".into())),
                ("da", Value::Str("bad".into())),
            ]),
        ] {
            assert!(data::insert(id, "qof", &fields).is_err());
        }
        let hostile = "x\\' OR 1=1 -- ";
        data::insert(id, "qof", &obj(&[("magac", Value::Str(hostile.into()))])).unwrap();
        let filter = obj(&[("da", Value::Null)]);
        assert_eq!(
            names(&data::find(id, "qof", Some(&filter)).unwrap()),
            [hostile]
        );
        assert!(names(
            &data::find(
                id,
                "qof",
                Some(&obj(&[("magac", Value::Str("other".into()))]))
            )
            .unwrap()
        )
        .is_empty());
        data::migrate(id).unwrap();
        assert_eq!(names(&data::find(id, "qof", None).unwrap()), [hostile]);
        data::close(id).unwrap();
        assert!(data::find(id, "qof", None).is_err());
    });
}

#[test]
fn database_handles_belong_to_their_execution_context() {
    let first = soplang::runtime::RuntimeContext::new();
    let second = soplang::runtime::RuntimeContext::new();
    let id = first.with(|| {
        let id = id_of(&data::open("sqlite", ":memory:", 2).unwrap());
        data::load_schema(id, SCHEMA).unwrap();
        data::migrate(id).unwrap();
        id
    });
    second.with(|| {
        assert!(data::find(id, "qof", None).is_err());
        assert!(data::close(id).is_err());
    });
    first.with(|| {
        let result = std::panic::catch_unwind(|| {
            data::transaction::<()>(id, || {
                data::insert(id, "qof", &obj(&[("magac", Value::Str("rollback".into()))]))?;
                panic!("exercise rollback after Rust panic");
            })
            .unwrap();
        });
        assert!(result.is_err());
        assert!(names(&data::find(id, "qof", None).unwrap()).is_empty());
        data::transaction(id, || {
            assert!(data::close(id).is_err());
            assert!(data::migrate(id).is_err());
            Ok(())
        })
        .unwrap();
    });
    drop(first);
    second.with(|| assert!(data::find(id, "qof", None).is_err()));
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
    let context = soplang::runtime::RuntimeContext::new();
    context.with(|| live_in_context(kind, target));
}

fn live_in_context(kind: &str, target: &str) {
    let handle = data::open(kind, target, 2).unwrap();
    let id = id_of(&handle);
    data::load_schema(id, SCHEMA).unwrap();
    data::migrate(id).unwrap();
    let marker = format!("live-{kind}-{}-Cali\\' OR 1=1 -- 🇸🇴", std::process::id());
    let failed = data::transaction::<()>(id, || {
        data::insert(id, "qof", &obj(&[("magac", Value::Str(marker.clone()))]))?;
        Err(soplang::error::runtime_error("rollback", 0, 0))
    });
    assert!(failed.is_err());
    let filter = obj(&[("magac", Value::Str(marker.clone()))]);
    assert!(names(&data::find(id, "qof", Some(&filter)).unwrap()).is_empty());
    data::transaction(id, || {
        data::insert(
            id,
            "qof",
            &obj(&[
                ("magac", Value::Str(marker.clone())),
                ("diyaar", Value::Bool(true)),
                ("qiime", Value::Float(2.5)),
            ]),
        )
    })
    .unwrap();
    // Migration must preserve the pre-existing row, including on MongoDB.
    data::migrate(id).unwrap();
    data::migrate(id).unwrap();
    let found = data::find(
        id,
        "qof",
        Some(&obj(&[("magac", Value::Str(marker.clone()))])),
    )
    .unwrap();
    assert_eq!(names(&found), [marker.clone()]);
    let Value::List(rows) = &found else {
        panic!("rows")
    };
    let Value::Object(row) = &rows.borrow()[0] else {
        panic!("row")
    };
    let row = row.borrow();
    assert!(matches!(row.get("id"), Some(Value::Int(_))));
    assert_eq!(row.get("diyaar"), Some(&Value::Bool(true)));
    assert_eq!(row.get("qiime"), Some(&Value::Float(2.5)));
    drop(row);
    data::update(id, "qof", &filter, &obj(&[("diyaar", Value::Bool(false))])).unwrap();
    assert_eq!(
        names(
            &data::find(
                id,
                "qof",
                Some(&obj(&[
                    ("magac", Value::Str(marker.clone())),
                    ("da", Value::Null)
                ]))
            )
            .unwrap()
        ),
        [marker.clone()]
    );
    data::delete(id, "qof", &obj(&[("magac", Value::Str(marker))])).unwrap();
    assert!(names(&data::find(id, "qof", Some(&filter)).unwrap()).is_empty());
    explicit_key_contract(id);
    migration_safety(id);
    data::close(id).unwrap();
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

#[test]
fn migrations_reject_incompatible_changes_before_ddl_and_keep_old_clients_readable() {
    let path = temp_db("migration_safety");
    let context = soplang::runtime::RuntimeContext::new();
    context.with(|| {
        let id = id_of(&data::open("sqlite", path.to_str().unwrap(), 2).unwrap());
        migration_safety(id);
        data::close(id).unwrap();
    });
    fs::remove_file(path).unwrap();
}

fn migration_safety(id: i64) {
    data::load_schema(id, SCHEMA).unwrap();
    data::migrate(id).unwrap();
    let marker = "migration-guard";
    data::insert(id, "qof", &obj(&[("magac", Value::Str(marker.into()))])).unwrap();
    // An allowed addition preceding an incompatible change must not leak into
    // non-transactional DDL connectors either: preflight covers every table.
    for bad in [
        SCHEMA.replace("magac qoraal", "magac abn"),
        SCHEMA.replace("magac qoraal", "magac qoraal?"),
        SCHEMA.replace("  da abn?\n", ""),
        SCHEMA.replace("id abn furaha", "other abn furaha"),
        SCHEMA.replace("}\n", "  required qoraal\n}\n"),
        "jadwal different {\n id abn furaha\n}\n".into(),
    ] {
        let bad = format!("jadwal not_created {{\n id abn furaha\n}}\n{bad}");
        data::load_schema(id, &bad).unwrap();
        let error = data::migrate(id).unwrap_err();
        assert_eq!(error.meta().code.as_deref(), Some("E059"), "{error}");
        data::load_schema(id, SCHEMA).unwrap();
        data::migrate(id).unwrap();
        assert!(names(&data::find(id, "qof", None).unwrap()).contains(&marker.into()));
    }
    let next = format!(
        "{}jadwal not_created {{\n id abn furaha\n}}\n",
        SCHEMA.replace("}\n", "  notes qoraal?\n}\n")
    );
    data::load_schema(id, &next).unwrap();
    data::migrate(id).unwrap();
    data::migrate(id).unwrap();
    // Old binaries can read their own declared projection during a rolling
    // application update, without running a migration that removes columns.
    data::load_schema(id, SCHEMA).unwrap();
    assert!(names(&data::find(id, "qof", None).unwrap()).contains(&marker.into()));
    assert!(data::migrate(id).is_err());
    data::load_schema(id, &next).unwrap();
    data::delete(id, "qof", &obj(&[("magac", Value::Str(marker.into()))])).unwrap();
}

#[test]
fn migration_journal_bootstraps_matching_legacy_hashes_and_refuses_corruption() {
    let path = temp_db("migration_journal");
    let context = soplang::runtime::RuntimeContext::new();
    context.with(|| {
        let id = id_of(&data::open("sqlite", path.to_str().unwrap(), 1).unwrap());
        data::load_schema(id, SCHEMA).unwrap();
        data::migrate(id).unwrap();
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection
            .execute_batch("DROP TABLE soplang_schema")
            .unwrap();
        let next = SCHEMA.replace("}\n", " extra qoraal?\n}\n");
        data::load_schema(id, &next).unwrap();
        assert!(data::migrate(id).is_err());
        data::load_schema(id, SCHEMA).unwrap();
        data::migrate(id).unwrap();
        data::load_schema(id, &next).unwrap();
        data::migrate(id).unwrap();
        let hash: String = connection
            .query_row("SELECT sha256 FROM soplang_guuri", [], |row| row.get(0))
            .unwrap();
        connection
            .execute(
                "UPDATE soplang_schema SET qaab = ?",
                ["jadwal wrong {\n id abn furaha\n}\n"],
            )
            .unwrap();
        assert!(data::migrate(id).is_err());
        assert_eq!(
            hash,
            connection
                .query_row::<String, _, _>("SELECT sha256 FROM soplang_guuri", [], |row| row.get(0))
                .unwrap()
        );
        data::close(id).unwrap();
    });
    fs::remove_file(path).unwrap();
}

#[test]
fn migrations_refuse_unmanaged_tables_and_add_required_columns_only_to_empty_tables() {
    let path = temp_db("migration_adoption");
    let context = soplang::runtime::RuntimeContext::new();
    context.with(|| {
        let id = id_of(&data::open("sqlite", path.to_str().unwrap(), 1).unwrap());
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection
            .execute_batch("CREATE TABLE qof (id INTEGER PRIMARY KEY, magac TEXT)")
            .unwrap();
        data::load_schema(id, SCHEMA).unwrap();
        assert!(data::migrate(id).is_err());
        connection.execute_batch("DROP TABLE qof").unwrap();
        data::migrate(id).unwrap();
        let next = SCHEMA.replace("}\n", " required qoraal\n}\n");
        data::load_schema(id, &next).unwrap();
        data::migrate(id).unwrap();
        data::insert(
            id,
            "qof",
            &obj(&[
                ("magac", Value::Str("Amina".into())),
                ("required", Value::Str("present".into())),
            ]),
        )
        .unwrap();
        data::close(id).unwrap();
    });
    fs::remove_file(path).unwrap();
}

#[test]
#[ignore]
fn live_mongo_schema_rows_and_recovered_counters() {
    use mongodb::{
        bson::{doc, Document},
        sync::Client,
    };
    let target = std::env::var("SOPLANG_MONGO").unwrap();
    let client = Client::with_uri_str(&target).unwrap();
    let name = format!(
        "soplang_contract_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let raw = client.database(&name);
    let mut url = url::Url::parse(&target).unwrap();
    url.set_path(&name);
    let schema = "jadwal entry {\n _id abn furaha\n amount jajab\n note qoraal?\n}\n";
    let context = soplang::runtime::RuntimeContext::new();
    context.with(|| {
        let id = id_of(&data::open("mongo", url.as_str(), 2).unwrap());
        data::load_schema(id, schema).unwrap();
        data::migrate(id).unwrap();
        data::insert(
            id,
            "entry",
            &obj(&[("_id", Value::Int(100)), ("amount", Value::Float(1.5))]),
        )
        .unwrap();
        let entries = raw.collection::<Document>("entry");
        entries
            .insert_one(doc! {"_id": 200_i32, "amount": 2_i32})
            .run()
            .unwrap();
        data::insert(id, "entry", &obj(&[("amount", Value::Float(3.5))])).unwrap();
        raw.collection::<Document>("soplang_counters")
            .delete_one(doc! {"_id":"entry"})
            .run()
            .unwrap();
        data::insert(id, "entry", &obj(&[("amount", Value::Float(4.5))])).unwrap();
        let Value::List(rows) = data::find(id, "entry", None).unwrap() else {
            panic!("rows")
        };
        let mut keys = Vec::new();
        for row in rows.borrow().iter() {
            let Value::Object(fields) = row else {
                panic!("row")
            };
            let fields = fields.borrow();
            let Some(Value::Int(key)) = fields.get("_id") else {
                panic!("declared _id")
            };
            keys.push(*key);
            assert!(matches!(fields.get("amount"), Some(Value::Float(_))));
            assert_eq!(fields.get("note"), Some(&Value::Null));
        }
        keys.sort();
        assert_eq!(keys, [100, 200, 201, 202]);
        entries
            .insert_one(doc! {"_id":300_i64, "amount":3.0, "extra":[1,2]})
            .bypass_document_validation(true)
            .run()
            .unwrap();
        let Value::List(rows) =
            data::find(id, "entry", Some(&obj(&[("_id", Value::Int(300))]))).unwrap()
        else {
            panic!("rows")
        };
        let Value::Object(fields) = &rows.borrow()[0] else {
            panic!("row")
        };
        assert!(!fields.borrow().contains_key("extra"));
        assert!(data::migrate(id).is_err());
        entries.delete_one(doc! {"_id":300_i64}).run().unwrap();
        entries
            .insert_one(doc! {"_id":400_i64, "amount":"invalid"})
            .bypass_document_validation(true)
            .run()
            .unwrap();
        assert!(data::find(id, "entry", None).is_err());
        assert!(data::migrate(id).is_err());
        entries.delete_one(doc! {"_id":400_i64}).run().unwrap();
        let next = schema.replace("}\n", " tag qoraal?\n}\n");
        data::load_schema(id, &next).unwrap();
        data::migrate(id).unwrap();
        let Value::List(rows) = data::find(id, "entry", None).unwrap() else {
            panic!("rows")
        };
        for row in rows.borrow().iter() {
            let Value::Object(fields) = row else {
                panic!("row")
            };
            assert_eq!(fields.borrow().get("tag"), Some(&Value::Null));
        }
        data::close(id).unwrap();
    });
    raw.drop().run().unwrap();
}

fn explicit_key_contract(id: i64) {
    let explicit = obj(&[
        ("id", Value::Int(-8128)),
        ("magac", Value::Str("explicit-key".into())),
    ]);
    data::insert(id, "qof", &explicit).unwrap();
    assert!(data::insert(id, "qof", &explicit).is_err());
    // Exercise both pooled connections after a failed explicit insertion.
    // A leaked IDENTITY_INSERT setting makes the generated insert fail.
    let generated: Vec<_> = (0..2)
        .map(|n| format!("generated-after-explicit-{n}"))
        .collect();
    for name in &generated {
        data::insert(id, "qof", &obj(&[("magac", Value::Str(name.clone()))])).unwrap();
    }
    for name in generated {
        data::delete(id, "qof", &obj(&[("magac", Value::Str(name))])).unwrap();
    }
    let filter = obj(&[("id", Value::Int(-8128))]);
    assert_eq!(
        names(&data::find(id, "qof", Some(&filter)).unwrap()),
        ["explicit-key"]
    );
    data::delete(id, "qof", &filter).unwrap();
    let rollback = data::transaction::<()>(id, || {
        data::insert(
            id,
            "qof",
            &obj(&[
                ("id", Value::Int(-8192)),
                ("magac", Value::Str("explicit-rollback".into())),
            ]),
        )?;
        Err(soplang::error::runtime_error("rollback", 0, 0))
    });
    assert!(rollback.is_err());
    let filter = obj(&[("id", Value::Int(-8192))]);
    assert!(names(&data::find(id, "qof", Some(&filter)).unwrap()).is_empty());
}

#[test]
fn sqlite_explicit_keys_recover_after_duplicates_and_rollbacks() {
    let context = soplang::runtime::RuntimeContext::new();
    context.with(|| {
        let id = id_of(&data::open("sqlite", ":memory:", 2).unwrap());
        data::load_schema(id, SCHEMA).unwrap();
        data::migrate(id).unwrap();
        explicit_key_contract(id);
        data::close(id).unwrap();
    });
}
