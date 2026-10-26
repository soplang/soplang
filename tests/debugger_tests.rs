//! Scripted source debugger sessions use real native frames in both backends.
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

struct Tree(PathBuf);
impl Tree {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "soplang_debug_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
}
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn session(command: &mut Command, input: &str) -> std::process::Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
const SOURCE: &str = "hawl inner(abn n) {\n door value = n + 1\n door nothing = maran\n celi value\n}\nhawl outer() {\n abn saved = 3\n celi inner(saved)\n}\ndoor result = outer()\nqor(result)\n";
#[test]
fn source_breakpoints_step_and_inspect_live_locals_in_jit_and_llvm() {
    let tree = Tree::new();
    let path = tree.0.join("source.sop");
    fs::write(&path, SOURCE).unwrap();
    let input =
        "deegaan\nxiga\ndeegaan\ntallaabo\neeg nothing\njir 1\neeg saved\nraad\nbax\nsoco\n";
    let mut command = Command::new(env!("CARGO_BIN_EXE_soplang"));
    command
        .arg("debug")
        .arg(&path)
        .args(["--soco", "--jebi", "2"]);
    let jit = session(&mut command, input);
    assert!(
        jit.status.success(),
        "{}",
        String::from_utf8_lossy(&jit.stderr)
    );
    assert_eq!(jit.stdout, b"4\n");
    let trace = String::from_utf8(jit.stderr).unwrap();
    for expected in [
        "n = 3",
        "value = <lama qeexin>",
        "value = 4",
        "nothing = maran",
        "saved = 3",
        "0: inner",
        "1: outer",
        "2: <main>",
    ] {
        assert!(trace.contains(expected), "missing {expected}: {trace}");
    }
    let binary = tree
        .0
        .join(format!("debug-bin{}", std::env::consts::EXE_SUFFIX));
    let built = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("debug")
        .arg(&path)
        .args(["--soco", "--jebi", "2", "--dhis"])
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let native = session(&mut Command::new(&binary), input);
    assert!(
        native.status.success(),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    assert_eq!(native.stdout, b"4\n");
    assert_eq!(String::from_utf8(native.stderr).unwrap(), trace);
}
#[test]
fn stepping_enters_calls_and_line_breakpoints_repeat_at_loop_headers() {
    let tree = Tree::new();
    let path = tree.0.join("loop.sop");
    fs::write(
        &path,
        "door n = 0\nintay (n < 2) {\n n = n + 1\n}\nqor(n)\n",
    )
    .unwrap();
    let output = session(
        Command::new(env!("CARGO_BIN_EXE_soplang"))
            .arg("debug")
            .arg(&path)
            .args(["--soco", "--jebi", "2"]),
        "eeg n\nsoco\neeg n\nsoco\neeg n\nsoco\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let trace = String::from_utf8(output.stderr).unwrap();
    for n in 0..=2 {
        assert!(trace.contains(&format!("n = {n}")), "{trace}");
    }
    assert_eq!(output.stdout, b"2\n");
    fs::write(&path, SOURCE).unwrap();
    let output = session(
        Command::new(env!("CARGO_BIN_EXE_soplang"))
            .arg("debug")
            .arg(&path),
        "tallaabo\neeg saved\nxiga\neeg saved\ntallaabo\neeg n\nsoco\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let trace = String::from_utf8(output.stderr).unwrap();
    assert!(
        trace.contains("saved = <lama qeexin>")
            && trace.contains("saved = 3")
            && trace.contains("n = 3"),
        "{trace}"
    );
}
#[test]
fn debugger_rejects_invalid_locations_options_and_closed_input() {
    let tree = Tree::new();
    let path = tree.0.join("main.sop");
    fs::write(&path, "// comment\nqor(1)\n").unwrap();
    for args in [
        vec!["--jebi", "1"],
        vec!["--jebi", "0"],
        vec!["--unknown"],
        vec!["--jebi"],
        vec!["--dhis"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
            .arg("debug")
            .arg(&path)
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
    let output = session(
        Command::new(env!("CARGO_BIN_EXE_soplang"))
            .arg("debug")
            .arg(&path),
        "",
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Gelintii baaristu way dhammaatay"));
    // Ordinary execution remains free of pauses and instrumentation.
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"1\n");
}
