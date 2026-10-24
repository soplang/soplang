//! Public CLI spellings share behavior across English, Somali, and -- forms.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "soplang_commands_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn cli(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_soplang"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn command_names_share_help_and_version_behavior() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (english, somali) in [
        ("init", "abuur"),
        ("run", "fuli"),
        ("build", "dhis"),
        ("check", "hubi"),
        ("fmt", "habee"),
        ("test", "tijaabi"),
        ("doc", "duk"),
        ("debug", "baar"),
        ("profile", "cabbir"),
        ("add", "kudar"),
        ("remove", "saar"),
        ("update", "cusboonaysii"),
        ("publish", "daabac"),
        ("data", "xog"),
        ("repl", "isdhexgal"),
        ("eval", "qiimee"),
        ("example", "tusaale"),
        ("inspect", "eeg"),
        ("lsp", "lsp"),
    ] {
        let expected = success(cli(root, &[english, "--help"]));
        assert!(expected.contains("Adeegsiga:"), "{english}");
        for alias in [
            somali.to_owned(),
            format!("--{english}"),
            format!("--{somali}"),
        ] {
            assert_eq!(
                success(cli(root, &[&alias, "--caawimo"])),
                expected,
                "{alias}"
            );
        }
    }
    let version = success(cli(root, &["version"]));
    for alias in ["nooc", "--version", "--nooc"] {
        assert_eq!(success(cli(root, &[alias])), version, "{alias}");
    }
    assert_eq!(
        success(cli(root, &["help"])),
        success(cli(root, &["caawimo"]))
    );
    assert!(!cli(root, &["docs"]).status.success());
}

#[test]
fn builds_files_and_project_with_names_and_checks_collisions() {
    let dir = temp_dir();
    fs::write(dir.join("a.sop"), "qor(\"A\")\n").unwrap();
    fs::write(dir.join("b.sop"), "qor(\"B\")\n").unwrap();
    success(cli(
        &dir,
        &[
            "--dhis",
            "a.sop",
            "b.sop",
            "--aamus",
            "--heer-hagaajin",
            "0",
        ],
    ));
    for (name, expected) in [("a", "A\n"), ("b", "B\n")] {
        let binary = dir
            .join("dhis")
            .join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
        let output = Command::new(binary).output().unwrap();
        assert_eq!(success(output), expected);
    }
    success(cli(&dir, &["dhis", "a.sop", "--magac", "app", "-q"]));
    assert!(dir
        .join("dhis")
        .join(format!("app{}", std::env::consts::EXE_SUFFIX))
        .is_file());
    fs::create_dir(dir.join("other")).unwrap();
    fs::write(dir.join("other/b.sop"), "qor(2)\n").unwrap();
    assert!(!cli(&dir, &["build", "b.sop", "other/b.sop"])
        .status
        .success());

    success(cli(&dir, &["abuur", "project", "--magac", "myapp"]));
    let project = dir.join("project");
    success(cli(&project, &["--build", "--quiet"]));
    assert!(project
        .join("dhis")
        .join(format!("myapp{}", std::env::consts::EXE_SUFFIX))
        .is_file());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn source_tools_accept_somali_actions_and_equivalent_options() {
    let dir = temp_dir();
    fs::write(dir.join("main.sop"), "qor(7)\n").unwrap();
    assert_eq!(success(cli(&dir, &["--fuli", "main.sop"])), "7\n");
    assert_eq!(success(cli(&dir, &["--qiimee", "qor(8)"])), "8\n");
    assert!(success(cli(&dir, &["hubi", "main.sop", "--kayd-keliya"])).contains("waa sax"));
    let document = dir.join("api.md");
    success(cli(&dir, &["--duk", "main.sop", "--natiijo", "api.md"]));
    assert!(document.is_file());
    assert!(success(cli(&dir, &["eeg", "main.sop", "--ast"])).contains("qor(7)"));
    fs::remove_dir_all(dir).unwrap();
}
