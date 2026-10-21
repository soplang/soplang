//! Base64 encode and decode (3.3 stage).

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
fn base64_round_trip_padding_and_url_mode() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
hubi(b64_qor("Man") == "TWFu", "three")
hubi(b64_qor("Ma") == "TWE=", "two")
hubi(b64_qor("M") == "TQ==", "one")
hubi(qoraal(b64_akhris("TWFu")) == "Man", "decode")
door raw = baytyo([251, 255])
hubi(b64_qor(raw) == "+/8=", "plus slash")
hubi(b64_qor(raw, "url") == "-_8", "url")
hubi(b64_akhris("-_8", "url").teed() == raw.teed(), "url decode")
qor("base64 ok")
"#;
    assert_eq!(run_program(source), "base64 ok\n");
}

#[test]
fn base64_failures_are_catchable() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door failed = been
tijaabi { b64_akhris("!!!!") } qabo (e) { failed = run }
hubi(failed, "alphabet")
failed = been
tijaabi { b64_qor("a", "foom") } qabo (e) { failed = run }
hubi(failed, "mode")
qor("base64 errors ok")
"#;
    assert_eq!(run_program(source), "base64 errors ok\n");
}
