//! Package manifest discovery while running a source file.

use std::fs;
use std::process::Command;

fn temp_dir(name: &str) -> std::path::PathBuf {
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

#[test]
fn a_valid_manifest_allows_the_program_to_run() {
    let dir = temp_dir("package_ok");
    fs::write(
        dir.join("soplang.toml"),
        "magac = \"xisaab\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\n",
    )
    .unwrap();
    fs::write(dir.join("main.sop"), "qor(\"package ok\")\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "package ok\n");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn an_incompatible_manifest_stops_the_program() {
    let dir = temp_dir("package_bad");
    fs::write(
        dir.join("soplang.toml"),
        "magac = \"xisaab\"\nnusxa = \"0.1.0\"\nsoplang = \"9\"\n",
    )
    .unwrap();
    fs::write(dir.join("main.sop"), "qor(\"no\")\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("Soplang 9"), "{stderr}");
    assert!(output.stdout.is_empty());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_local_path_dependency_must_match_the_package_name() {
    let root = temp_dir("package_dep");
    let lib = root.join("xisaab");
    let app = root.join("app");
    fs::create_dir_all(&lib).unwrap();
    fs::create_dir_all(&app).unwrap();
    fs::write(
        lib.join("soplang.toml"),
        "magac = \"xisaab\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\n",
    )
    .unwrap();
    fs::write(
        app.join("soplang.toml"),
        "magac = \"app\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\n\n[ku-tiirsan]\nxisaab = \"../xisaab\"\n",
    )
    .unwrap();
    fs::write(app.join("main.sop"), "qor(\"dep ok\")\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "dep ok\n");

    fs::write(
        lib.join("soplang.toml"),
        "magac = \"kale\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("kale"), "{stderr}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_lockfile_must_match_the_resolved_dependencies() {
    let root = temp_dir("package_lock");
    let lib = root.join("xisaab");
    let app = root.join("app");
    fs::create_dir_all(&lib).unwrap();
    fs::create_dir_all(&app).unwrap();
    fs::write(
        lib.join("soplang.toml"),
        "magac = \"xisaab\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\n",
    )
    .unwrap();
    fs::write(
        app.join("soplang.toml"),
        "magac = \"app\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\n\n[ku-tiirsan]\nxisaab = \"../xisaab\"\n",
    )
    .unwrap();
    fs::write(app.join("main.sop"), "qor(\"lock ok\")\n").unwrap();
    fs::write(
        app.join("soplang.lock"),
        "# soplang.lock\n\n[[xirmo]]\nmagac = \"xisaab\"\nnusxa = \"0.1.0\"\njid = \"../xisaab\"\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    fs::write(
        app.join("soplang.lock"),
        "# soplang.lock\n\n[[xirmo]]\nmagac = \"xisaab\"\nnusxa = \"9.9.9\"\njid = \"../xisaab\"\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("soplang.lock"), "{stderr}");
    let _ = fs::remove_dir_all(root);
}
