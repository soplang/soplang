//! Run every public example and compare its exit status and standard output.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn examples() -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir("examples")
        .expect("examples directory")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sop"))
        .filter(|path| !path.file_stem().unwrap().to_string_lossy().starts_with('_'))
        .collect();
    paths.sort();
    paths
}

fn command_output(mut command: Command, input: Option<&str>) -> Output {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command.spawn().expect("start example");
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    child.wait_with_output().expect("finish example")
}

fn check_output(path: &Path, output: Output) {
    assert!(
        output.status.success(),
        "{} failed: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{} wrote stderr: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let expected_path = path.with_extension("expected");
    let expected =
        fs::read_to_string(&expected_path).expect("every public example must have expected output");
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\r\n", "\n"),
        expected.replace("\r\n", "\n"),
        "{} stdout",
        path.display()
    );
}

#[test]
fn test_each_example_against_expected() {
    for path in examples() {
        let mut command = Command::new(env!("CARGO_BIN_EXE_soplang"));
        command.arg("--no-color");
        if path.file_stem().unwrap() == "23_strict_types" {
            command.arg("--strict");
        }
        command.arg(&path);
        let input = (path.file_stem().unwrap() == "13_input").then_some("TestUser\n");
        check_output(&path, command_output(command, input));
    }
}

#[test]
fn test_aot_each_example_against_expected() {
    let dir = std::env::temp_dir().join(format!("soplang_examples_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    for path in examples() {
        let binary = dir
            .join(path.file_stem().unwrap())
            .with_extension(std::env::consts::EXE_EXTENSION);
        let mut build = Command::new(env!("CARGO_BIN_EXE_soplang"));
        build.args(["--no-color", "--build", "--quiet"]);
        if path.file_stem().unwrap() == "23_strict_types" {
            build.arg("--strict");
        }
        let result = build.arg(&path).arg("-o").arg(&binary).output().unwrap();
        assert!(
            result.status.success(),
            "{} build: {}",
            path.display(),
            String::from_utf8_lossy(&result.stderr)
        );
        let input = (path.file_stem().unwrap() == "13_input").then_some("TestUser\n");
        let mut command = Command::new(&binary);
        command.env("NO_COLOR", "1");
        check_output(&path, command_output(command, input));
        fs::remove_file(binary).unwrap();
    }
    fs::remove_dir(dir).unwrap();
}
