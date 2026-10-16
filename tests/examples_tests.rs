//! Integration tests: run example files, assert stdout.
//! 7.1: ad-hoc example checks. 7.2: each example vs .expected file.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run_file(path: &Path) -> String {
    run_file_with_stdin(path, None)
}

/// Run a .sop file, optionally with stdin (e.g. for gelin() in 13_input.sop).
fn run_file_with_stdin(path: &Path, stdin: Option<&str>) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let mut cmd = Command::new(exe);
    cmd.arg(path);
    if let Some(input) = stdin {
        use std::io::Write;
        cmd.stdin(std::process::Stdio::piped());
        let mut child = cmd.spawn().expect("spawn soplang");
        if let Some(mut stdin_write) = child.stdin.take() {
            stdin_write.write_all(input.as_bytes()).expect("write stdin");
        }
        let out = child.wait_with_output().expect("wait soplang");
        return String::from_utf8(out.stdout).unwrap();
    }
    let out = cmd.output().expect("run soplang");
    String::from_utf8(out.stdout).unwrap()
}

/// Path to .expected for a .sop file (e.g. examples/15_classes.sop -> examples/15_classes.expected).
fn expected_path(sop_path: &Path) -> PathBuf {
    sop_path
        .with_file_name(sop_path.file_stem().unwrap_or_default().to_string_lossy().to_string() + ".expected")
}

#[test]
fn test_example_hello() {
    let path = Path::new("examples/01_hello.sop");
    let output = run_file(path);
    assert_eq!(output, "Salaan, Adduunka!\n");
}

/// Phase 7.2: Run each examples/*.sop that has a .expected file; assert stdout matches.
/// Regenerate .expected: see examples/README.md.
/// Skips the random example and any .sop without a .expected file.
#[test]
fn test_each_example_against_expected() {
    let examples_dir = Path::new("examples");
    let dir = match fs::read_dir(examples_dir) {
        Ok(d) => d,
        Err(_) => return,
    };
    let mut sop_paths: Vec<PathBuf> = dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "sop").unwrap_or(false))
        .collect();
    sop_paths.sort();

    for sop_path in sop_paths {
        let stem = sop_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem == "14_random" {
            continue; // non-deterministic (xul)
        }
        let expected_path = expected_path(&sop_path);
        if !expected_path.exists() {
            continue;
        }
        let expected = fs::read_to_string(&expected_path).unwrap_or_default();
        // 13_input.sop uses gelin(); feed stdin so output is deterministic.
        let actual = if stem == "13_input" {
            run_file_with_stdin(&sop_path, Some("TestUser\n"))
        } else {
            run_file(&sop_path)
        };
        assert_eq!(
            actual,
            expected,
            "example {} stdout did not match {}",
            sop_path.display(),
            expected_path.display()
        );
    }
}
