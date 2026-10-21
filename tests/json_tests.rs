//! JSON parse and generation (3.3 stage).

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
fn json_round_trip_preserves_plain_objects_and_class_keys() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door text = e"{\"__class__\":\"Bisad\",\"da\":7,\"fur\":true,\"madhan\":null,\"teed\":[1,1.5]}"
door value = json_akhris(text)
hubi(nooc(value) == "walax", "plain object")
hubi(value.__class__ == "Bisad", "class key")
hubi(value.da == 7 && value.fur && value.madhan == maran, "fields")
hubi(nooc(value.teed[0]) == "abn" && nooc(value.teed[1]) == "jajab", "numbers")
hubi(json_qor(value) == e"{\"__class__\":\"Bisad\",\"da\":7,\"fur\":true,\"madhan\":null,\"teed\":[1,1.5]}", "stable keys")
qor("json ok")
"#;
    assert_eq!(run_program(source), "json ok\n");
}

#[test]
fn json_failures_are_catchable() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door failed = been
tijaabi { json_akhris("{") } qabo (e) { failed = run }
hubi(failed, "parse")
failed = been
tijaabi { json_qor(baytyo([1])) } qabo (e) { failed = run }
hubi(failed, "bytes")
qor("json errors ok")
"#;
    assert_eq!(run_program(source), "json errors ok\n");
}
