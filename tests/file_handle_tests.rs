//! Explicit file handles, seeking, and atomic replacement.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "soplang_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn sop_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn run_in(dir: &Path, source: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .current_dir(dir)
        .arg("-c")
        .arg(source)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success(), "{stderr}\n{stdout}");
    stdout
}

#[test]
fn handles_seek_lines_and_atomic_replace() {
    let dir = temp_dir("file_handles");
    let note = sop_path(&dir.join("note.txt"));
    let temp = sop_path(&dir.join(".note.txt.sopliin"));
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
door f = fayl_fur("{note}", "qor")
hubi(f.nooc == "fayl" && f.hab == "qor", "mode")
fayl_qor(f, e"a\nb\n")
fayl_nadiifi(f)
fayl_raad(f, 0, "bilow")
hubi(fayl_xariiq(f) == "a", "line")
hubi(fayl_xariiq(f) == "b", "line 2")
hubi(fayl_xariiq(f) == maran, "eof")
fayl_xidh(f)
door append = fayl_fur("{note}", "kudar")
fayl_raad(append, 0, "bilow")
fayl_qor(append, "c")
fayl_xidh(append)
hubi(akhrifayl("{note}") == e"a\nb\nc", "append")
door read = fayl_fur("{note}", "akhris")
fayl_raad(read, 2, "bilow")
hubi(qoraal(fayl_akhris(read, 1)) == "b", "chunk")
hubi(fayl_meel(read) == 3, "position")
fayl_xidh(read)
door closed = been
tijaabi {{ fayl_akhris(read, 1) }} qabo (e) {{ closed = run }}
hubi(closed, "closed")
fayl_beddel("{note}", "cusub")
hubi(akhrifayl("{note}") == "cusub", "replace")
hubi(!fayl_jiraa("{temp}"), "temp gone")
qor("handles ok")
"#
    );
    assert_eq!(run_in(&dir, &source), "handles ok\n");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn file_handles_build_with_llvm() {
    let dir = temp_dir("file_handle_llvm");
    let main = dir.join("main.sop");
    fs::write(
        &main,
        "fayl_beddel(\"note.txt\", \"llvm\")\nqor(akhrifayl(\"note.txt\"))\n",
    )
    .unwrap();
    let binary = dir.join("file-bin");
    let built = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .current_dir(&dir)
        .arg("--build")
        .arg(&main)
        .arg("-o")
        .arg(&binary)
        .arg("-q")
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let output = Command::new(&binary).current_dir(&dir).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "llvm\n");
    let _ = fs::remove_dir_all(dir);
}
