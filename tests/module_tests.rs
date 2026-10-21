//! Namespaced file imports (3.4 stage).

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
fn namespaced_import_hides_private_names_and_keeps_flat_imports() {
    let dir = temp_dir("module");
    fs::write(
        dir.join("math.sop"),
        "soosaar wad, base\ndoor base = 10\nhawl wad(a, b) { celi a + b + base }\nhawl kale() { celi 1 }\nhawl _hidden() { celi 1 }\n",
    )
    .unwrap();
    fs::write(
        dir.join("main.sop"),
        r#"
keen "math.sop" sida xisaab
hawl hubi(v, name) { haddii (!v) { tuur name } }
door add = xisaab.wad
hubi(add(xisaab.base, 2) == 22, "call")
door failed = been
tijaabi { wad(1, 1) } qabo (e) { failed = run }
hubi(failed, "hidden function")
failed = been
tijaabi { xisaab._hidden() } qabo (e) { failed = run }
hubi(failed, "private")
failed = been
tijaabi { xisaab.kale() } qabo (e) { failed = run }
hubi(failed, "unlisted")
qor("module ok")
"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "module ok\n");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn namespaced_import_builds_with_llvm() {
    let dir = temp_dir("module_llvm");
    fs::write(dir.join("math.sop"), "hawl wad(a, b) { celi a + b }\n").unwrap();
    let main = dir.join("main.sop");
    fs::write(&main, "keen \"math.sop\" sida xisaab\nqor(xisaab.wad(2, 3))\n").unwrap();
    let binary = dir.join("module-bin");
    let built = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("--build")
        .arg(&main)
        .arg("-o")
        .arg(&binary)
        .arg("-q")
        .output()
        .unwrap();
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let output = Command::new(&binary).output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "5\n");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn unknown_export_is_rejected() {
    let dir = temp_dir("module_export");
    fs::write(dir.join("math.sop"), "soosaar maqan\nhawl wad(a, b) { celi a + b }\n").unwrap();
    fs::write(dir.join("main.sop"), "keen \"math.sop\" sida xisaab\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("maqan"), "{stderr}");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn selective_import_binds_only_requested_names() {
    let dir = temp_dir("module_select");
    fs::write(
        dir.join("math.sop"),
        "soosaar wad, base\ndoor base = 10\nhawl wad(a, b) { celi a + b }\nhawl kale() { celi 9 }\n",
    )
    .unwrap();
    fs::write(
        dir.join("main.sop"),
        r#"
keen "math.sop" ka wad sida add, base
hawl hubi(v, name) { haddii (!v) { tuur name } }
hubi(add(base, 2) == 12, "selected")
door failed = been
tijaabi { wad(1, 1) } qabo (e) { failed = run }
hubi(failed, "original name")
failed = been
tijaabi { kale() } qabo (e) { failed = run }
hubi(failed, "unimported")
qor("select ok")
"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "select ok\n");
    let _ = fs::remove_dir_all(dir);
}
