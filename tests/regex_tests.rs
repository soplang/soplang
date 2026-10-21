//! Regular-expression search and replacement (3.3 stage).

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
fn regex_find_all_and_replace() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
hubi(regex_raadi("a+", "baaac") == "aaa", "first")
hubi(regex_raadi("z", "abc") == maran, "none")
door all = regex_dhammaan(e"\\d+", "a1b22")
hubi(dherer(all) == 2 && all[0] == "1" && all[1] == "22", "all")
hubi(regex_beddel(e"\\d+", "a1b22", "n") == "anbn", "replace")
hubi(regex_beddel("(a)(b)", "ab", "$2$1") == "ba", "groups")
qor("regex ok")
"#;
    assert_eq!(run_program(source), "regex ok\n");
}

#[test]
fn regex_invalid_patterns_are_catchable() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door failed = been
tijaabi { regex_raadi("(", "abc") } qabo (e) { failed = run }
hubi(failed, "pattern")
qor("regex errors ok")
"#;
    assert_eq!(run_program(source), "regex errors ok\n");
}
