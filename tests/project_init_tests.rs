use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "soplang_init_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn init(path: &Path, extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("init")
        .arg(path)
        .args(extra)
        .output()
        .unwrap()
}

fn assert_program(path: &Path, expected: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    for opt in [0, 2] {
        let binary = path.with_file_name(format!("starter-{opt}{}", std::env::consts::EXE_SUFFIX));
        let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
            .arg("--build")
            .arg(path)
            .arg("--opt-level")
            .arg(opt.to_string())
            .arg("-o")
            .arg(&binary)
            .arg("-q")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = Command::new(binary).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
}

#[test]
fn application_starter_runs_through_both_backends() {
    let dir = temp();
    let project = dir.join("My Project");
    let output = init(&project, &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest = soplang::package::parse_manifest(
        &fs::read_to_string(project.join("soplang.toml")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest.name, "my-project");
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.soplang, env!("CARGO_PKG_VERSION"));
    assert_program(&project.join("main.sop"), "Salaan, adduun!\n");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn library_starter_is_importable_as_a_local_dependency() {
    let dir = temp();
    let library = dir.join("library");
    let app = dir.join("app");
    assert!(init(&library, &["--lib", "--name", "salaan"])
        .status
        .success());
    assert!(library.join("lib.sop").is_file());
    assert!(!library.join("main.sop").exists());
    assert!(init(&app, &[]).status.success());
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .current_dir(&app)
        .args(["--ku-dar", "salaan", "--qeexid", "../library"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::write(
        app.join("main.sop"),
        "keen \"salaan\" sida maktabad\nqor(maktabad.salaan())\n",
    )
    .unwrap();
    assert_program(&app.join("main.sop"), "Salaan, adduun!\n");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn initialization_preserves_existing_files_and_rejects_invalid_names() {
    let dir = temp();
    let project = dir.join("app");
    fs::create_dir(&project).unwrap();
    fs::write(project.join("main.sop"), "keep source").unwrap();
    let output = init(&project, &[]);
    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(project.join("main.sop")).unwrap(),
        "keep source"
    );
    assert!(!project.join("soplang.toml").exists());
    fs::remove_file(project.join("main.sop")).unwrap();
    fs::write(project.join("soplang.toml"), "keep manifest").unwrap();
    assert!(!init(&project, &[]).status.success());
    assert_eq!(
        fs::read_to_string(project.join("soplang.toml")).unwrap(),
        "keep manifest"
    );
    assert!(!project.join("main.sop").exists());
    let invalid = dir.join("invalid");
    assert!(!init(&invalid, &["--name", "bad/name"]).status.success());
    assert!(!invalid.exists());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn initialization_supports_current_directory_and_somali_help() {
    let dir = temp();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .current_dir(&dir)
        .args(["init", "--name", "tusaale"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.join("main.sop").is_file());
    let help = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .args(["init", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("Adeegsiga:") && help.contains("--lib") && !help.contains("Print help"));
    let bad = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .args(["init", "--unknown"])
        .output()
        .unwrap();
    assert!(!bad.status.success());
    let error = String::from_utf8(bad.stderr).unwrap();
    assert!(
        error.contains("lama aqoonsan")
            && error.contains("soplang init")
            && !error.contains("unexpected argument")
    );
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn initialization_does_not_replace_dangling_starter_symlinks() {
    let dir = temp();
    std::os::unix::fs::symlink(dir.join("missing"), dir.join("lib.sop")).unwrap();
    assert!(!init(&dir, &["--lib", "--name", "safe"]).status.success());
    assert!(fs::symlink_metadata(dir.join("lib.sop"))
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(!dir.join("missing").exists() && !dir.join("soplang.toml").exists());
    fs::remove_dir_all(dir).unwrap();
}
