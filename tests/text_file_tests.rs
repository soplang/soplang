//! UTF-8 text file builtins (3.3 initial stage).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp_workdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "soplang_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn sop_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn run_program_in(source: &str, dir: &Path) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = Command::new(exe)
        .current_dir(dir)
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
fn text_file_read_write_and_append_round_trip() {
    let dir = temp_workdir("text_round_trip");
    let file = dir.join("note.txt");
    let path = sop_path(&file);
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
qor_fayl("{path}", "hal")
door text = akhrifayl("{path}")
hubi(text == "hal", "read")
kudar_fayl("{path}", " laba")
hubi(akhrifayl("{path}") == "hal laba", "append")
qor("text files ok")
"#
    );
    assert_eq!(
        run_program_in(&source, &dir),
        "text files ok\n",
        "fixture at {}",
        file.display()
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_text_file_errors_are_catchable() {
    let dir = temp_workdir("text_missing");
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door failed = been
tijaabi {
    akhrifayl("missing-note.txt")
} qabo (e) {
    failed = run
}
hubi(failed, "catch")
qor("missing ok")
"#;
    assert_eq!(run_program_in(source, &dir), "missing ok\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn invalid_utf8_text_file_is_reported_on_read() {
    let dir = temp_workdir("text_utf8");
    let file = dir.join("bad.txt");
    fs::write(&file, b"\xff\xfe").unwrap();
    let path = sop_path(&file);
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
door failed = been
tijaabi {{
    akhrifayl("{path}")
}} qabo (e) {{
    failed = run
}}
hubi(failed, "utf8")
qor("utf8 ok")
"#
    );
    assert_eq!(run_program_in(&source, &dir), "utf8 ok\n");
    let _ = fs::remove_dir_all(&dir);
}
