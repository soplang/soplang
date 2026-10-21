//! Whole-file binary read and write (3.3 stage).

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
fn binary_file_round_trip_and_append() {
    let dir = temp_workdir("binary_round_trip");
    let file = dir.join("raw.bin");
    let path = sop_path(&file);
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
door data = baytyo([0, 255, 1])
qor_baytyo("{path}", data)
door again = akhrif_baytyo("{path}")
hubi(again.teed() == data.teed(), "read")
kudar_baytyo("{path}", baytyo([2]))
door full = akhrif_baytyo("{path}")
hubi(dherer(full) == 4 && full[3] == 2, "append")
qor("binary files ok")
"#
    );
    assert_eq!(run_program_in(&source, &dir), "binary files ok\n");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn binary_write_requires_bytes() {
    let dir = temp_workdir("binary_type");
    let file = dir.join("typed.bin");
    let path = sop_path(&file);
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
door failed = been
tijaabi {{
    qor_baytyo("{path}", "text")
}} qabo (e) {{
    failed = run
}}
hubi(failed, "type")
qor("binary type ok")
"#
    );
    assert_eq!(run_program_in(&source, &dir), "binary type ok\n");
    let _ = fs::remove_dir_all(&dir);
}
