//! URL percent-encoding (3.3 stage).

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
fn url_component_and_form_round_trip() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door text = "Ayaan Cali/é"
hubi(url_qor(text) == "Ayaan%20Cali%2F%C3%A9", "component")
hubi(url_akhris(url_qor(text)) == text, "decode")
hubi(url_qor("Ayaan Cali", "foom") == "Ayaan+Cali", "form")
hubi(url_akhris("Ayaan+Cali", "foom") == "Ayaan Cali", "form decode")
hubi(url_akhris("Ayaan+Cali") == "Ayaan+Cali", "plus stays")
qor("url ok")
"#;
    assert_eq!(run_program(source), "url ok\n");
}

#[test]
fn url_decode_failures_are_catchable() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door failed = been
tijaabi { url_akhris("%ZZ") } qabo (e) { failed = run }
hubi(failed, "hex")
failed = been
tijaabi { url_qor("a", "kale") } qabo (e) { failed = run }
hubi(failed, "mode")
qor("url errors ok")
"#;
    assert_eq!(run_program(source), "url errors ok\n");
}
