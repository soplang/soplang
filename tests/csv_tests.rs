//! CSV parse and generation (3.3 stage).

use std::process::Command;

fn run_program(source: &str) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = Command::new(exe)
        .arg("-c")
        .arg(source)
        .output()
        .expect("run soplang");
    assert!(
        out.status.success(),
        "program failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn csv_round_trip_quotes_and_numbers() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door rows = csv_akhris(e"magac,da\n\"Ayaan, Cali\",7\n")
hubi(dherer(rows) == 2, "rows")
hubi(rows[1][0] == "Ayaan, Cali" && rows[1][1] == "7", "quoted")
door written = csv_qor([["magac", "da"], ["Ayaan, Cali", 7]])
hubi(written == e"magac,da\n\"Ayaan, Cali\",7", "write")
qor("csv ok")
"#;
    assert_eq!(run_program(source), "csv ok\n");
}

#[test]
fn csv_failures_are_catchable() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door failed = been
tijaabi { csv_akhris(e"\"open") } qabo (e) { failed = run }
hubi(failed, "parse")
failed = been
tijaabi { csv_qor([[baytyo([1])]]) } qabo (e) { failed = run }
hubi(failed, "type")
qor("csv errors ok")
"#;
    assert_eq!(run_program(source), "csv errors ok\n");
}
