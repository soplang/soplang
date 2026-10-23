//! Live verified-TLS connector tests. Each test needs a disposable database
//! whose server certificate is signed by a test CA for DNS:localhost only.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

use soplang::data::{self, ConnectionOptions};
use soplang::Value;

const SCHEMA: &str = "jadwal qof {\n id abn furaha\n magac qoraal\n}\n";

fn object(fields: &[(&str, Value)]) -> Value {
    Value::Object(Rc::new(RefCell::new(HashMap::from_iter(
        fields
            .iter()
            .map(|(key, value)| (key.to_string(), value.clone())),
    ))))
}

fn handle_id(value: Value) -> i64 {
    let Value::Object(fields) = value else {
        panic!("database handle must be an object");
    };
    let fields = fields.borrow();
    let Some(Value::Int(id)) = fields.get("aqoonsi") else {
        panic!("database handle has no integer ID");
    };
    *id
}

fn live_tls(kind: &str, target_env: &str) {
    let target = std::env::var(target_env).expect(target_env);
    assert!(target.contains("localhost"), "target must use localhost");
    let ca = PathBuf::from(std::env::var("SOPLANG_TLS_CA").expect("SOPLANG_TLS_CA"));
    let wrong_ca = PathBuf::from(std::env::var("SOPLANG_TLS_BAD_CA").expect("SOPLANG_TLS_BAD_CA"));
    let verified = ConnectionOptions {
        tls: true,
        ca_file: Some(ca),
    };
    let untrusted = ConnectionOptions {
        tls: true,
        ca_file: None,
    };
    let bad_ca = ConnectionOptions {
        tls: true,
        ca_file: Some(wrong_ca),
    };
    let wrong_host = target.replace("localhost", "127.0.0.1");

    let context = soplang::runtime::RuntimeContext::new();
    context.with(|| {
        assert!(data::open_with_options(kind, &target, 1, &untrusted).is_err());
        assert!(data::open_with_options(kind, &target, 1, &bad_ca).is_err());
        assert!(data::open_with_options(kind, &wrong_host, 1, &verified).is_err());

        let id = handle_id(data::open_with_options(kind, &target, 2, &verified).unwrap());
        data::load_schema(id, SCHEMA).unwrap();
        data::migrate(id).unwrap();
        let marker = format!(
            "tls-{kind}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let row = object(&[("magac", Value::Str(marker))]);
        data::insert(id, "qof", &row).unwrap();
        let rows = data::find(id, "qof", Some(&row)).unwrap();
        let Value::List(rows) = rows else {
            panic!("query must return rows");
        };
        assert_eq!(rows.borrow().len(), 1);
        data::close(id).unwrap();
    });
}

fn postgres_language_apis() {
    let target = std::env::var("SOPLANG_TLS_POSTGRES").unwrap();
    let ca = std::env::var("SOPLANG_TLS_CA").unwrap();
    let dir = std::env::temp_dir().join(format!(
        "soplang_tls_apis_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let generated = data::generate_client(&data::parse_schema(SCHEMA).unwrap()).unwrap();
    fs::write(dir.join("client.sop"), generated).unwrap();
    let target = serde_json::to_string(&target).unwrap();
    let ca = serde_json::to_string(&ca).unwrap();
    let source = format!(
        "keen \"xog\" sida x\n\
         keen \"client.sop\" sida c\n\
         door native = xog_fur(\"postgres\", {target}, {{ tls: run, shahaado_ca: {ca} }})\n\
         xog_xidh(native)\n\
         door bundled = x.fur(\"postgres\", {target}, tls: run, shahaado_ca: {ca})\n\
         x.xidh(bundled)\n\
         door typed = cusub c.MacmiilXog(\"postgres\", {target}, tls: run, shahaado_ca: {ca})\n\
         typed.xidh()\n\
         qor(\"tls APIs ok\")\n"
    );
    let main = dir.join("main.sop");
    fs::write(&main, &source).unwrap();
    let jit = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(&main)
        .output()
        .unwrap();
    assert!(
        jit.status.success(),
        "{}",
        String::from_utf8_lossy(&jit.stderr)
    );
    assert_eq!(jit.stdout, b"tls APIs ok\n");
    let binary = dir.join(format!("tls-client{}", std::env::consts::EXE_SUFFIX));
    soplang::build_source(&source, Some(&main), &binary, 2, false).unwrap();
    let llvm = Command::new(binary).output().unwrap();
    assert!(
        llvm.status.success(),
        "{}",
        String::from_utf8_lossy(&llvm.stderr)
    );
    assert_eq!(llvm.stdout, b"tls APIs ok\n");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
#[ignore = "requires disposable PostgreSQL TLS server and SOPLANG_TLS_* variables"]
fn live_tls_postgres() {
    live_tls("postgres", "SOPLANG_TLS_POSTGRES");
    postgres_language_apis();
}

#[test]
#[ignore = "requires disposable CockroachDB TLS server and SOPLANG_TLS_* variables"]
fn live_tls_cockroach() {
    live_tls("cockroach", "SOPLANG_TLS_COCKROACH");
}

#[test]
#[ignore = "requires disposable MySQL TLS server and SOPLANG_TLS_* variables"]
fn live_tls_mysql() {
    live_tls("mysql", "SOPLANG_TLS_MYSQL");
}

#[test]
#[ignore = "requires disposable MariaDB TLS server and SOPLANG_TLS_* variables"]
fn live_tls_mariadb() {
    live_tls("mariadb", "SOPLANG_TLS_MARIADB");
}
