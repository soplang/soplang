//! Package command forms and generated API documentation share compiler rules.
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
struct Tree(PathBuf);
impl Tree {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "soplang_project_tools_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn cli(tree: &Tree, args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .current_dir(&tree.0)
        .args(args)
        .output()
        .unwrap();
    if !output.status.success() {
        eprintln!(
            "soplang {:?}: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output
}
fn ok(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
#[test]
fn api_docs_preserve_public_signatures_defaults_comments_and_original_source() {
    let tree = Tree::new();
    let path = tree.0.join("lib.sop");
    let source = r#"soosaar Qof, salaan, count
/// Count of attempts.
madoor abn count = 2
/// Send a greeting.
/// Defaults retain their source spelling.
hawl salaan(qoraal name = r"a\b", teed[qoraal] *rest, walax[qoraal, qoraal] **labels): qoraal { celi name }
/// A public person.
qaab Qof {
    /// Construct a person.
    hawl dhis(qoraal name): maran { nafta.name = name }
    /// Read its name.
    hawl hel(): qoraal { celi nafta.name }
    hawl _private() { celi 1 }
}
hawl hidden() { celi 9 }
tijaabo "expected" qalad("E030") { qor(1 / 0) }
"#;
    fs::write(&path, source).unwrap();
    let docs = ok(cli(&tree, &["doc", "lib.sop"]));
    for expected in [
        "Defaults retain their source spelling.",
        r#"hawl salaan(qoraal name = r"a\b", teed[qoraal] *rest, walax[qoraal, qoraal] **labels): qoraal"#,
        "hawl dhis(qoraal name): maran",
        "hawl hel(): qoraal",
        "Count of attempts.",
        "madoor abn count",
        r#"tijaabo "expected" qalad("E030")"#,
    ] {
        assert!(docs.contains(expected), "{expected}: {docs}");
    }
    assert!(!docs.contains("hawl hidden") && !docs.contains("hawl _private"));
    let all = ok(cli(&tree, &["doc", "lib.sop", "--gaar"]));
    assert!(all.contains("hawl hidden") && all.contains("hawl _private"));
    ok(cli(&tree, &["doc", "lib.sop", "--qor", "api.md"]));
    assert_eq!(fs::read_to_string(tree.0.join("api.md")).unwrap(), docs);
    assert!(!cli(&tree, &["doc", "lib.sop", "--qor", "lib.sop"])
        .status
        .success());
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    fs::write(&path, "hawl broken(").unwrap();
    assert!(!cli(&tree, &["doc", "lib.sop", "--qor", "api.md"])
        .status
        .success());
    assert_eq!(fs::read_to_string(tree.0.join("api.md")).unwrap(), docs);
}
#[test]
fn run_build_and_dependency_commands_respect_project_roots_and_options() {
    let tree = Tree::new();
    fs::create_dir_all(tree.0.join("app/src/nested")).unwrap();
    fs::create_dir_all(tree.0.join("helper")).unwrap();
    fs::write(
        tree.0.join("app/soplang.toml"),
        "magac = 'app'\nnusxa = '0.1.0'\nsoplang = '3'\ngelid = 'src'\n",
    )
    .unwrap();
    fs::write(
        tree.0.join("helper/soplang.toml"),
        "magac = 'helper'\nnusxa = '0.1.0'\nsoplang = '3'\n",
    )
    .unwrap();
    fs::write(
        tree.0.join("helper/lib.sop"),
        "hawl value(): abn { celi 7 }\n",
    )
    .unwrap();
    fs::write(tree.0.join("app/src/main.sop"), "qor(amarro())\n").unwrap();
    let out = ok(cli(
        &tree,
        &[
            "run",
            "app/src/nested",
            "--adag",
            "--offline",
            "--",
            "alpha",
            "--beta",
        ],
    ));
    assert!(out.contains("alpha") && out.contains("--beta"), "{out}");
    ok(cli(
        &tree,
        &["add", "helper", "../helper", "app", "--offline"],
    ));
    let manifest = fs::read_to_string(tree.0.join("app/soplang.toml")).unwrap();
    assert!(manifest.contains("helper"));
    ok(cli(&tree, &["update", "app", "--offline"]));
    fs::write(
        tree.0.join("app/src/main.sop"),
        "keen \"helper\" sida h\nqor(h.value())\n",
    )
    .unwrap();
    assert_eq!(ok(cli(&tree, &["run", "app", "--offline"])), "7\n");
    let binary = tree
        .0
        .join(format!("built{}", std::env::consts::EXE_SUFFIX));
    let output = ok(cli(
        &tree,
        &[
            "build",
            "app",
            "--offline",
            "-O",
            "0",
            "-o",
            binary.to_str().unwrap(),
            "-q",
        ],
    ));
    assert!(output.is_empty());
    assert_eq!(ok(Command::new(&binary).output().unwrap()), "7\n");
    ok(cli(&tree, &["remove", "helper", "app", "--offline"]));
    assert!(!fs::read_to_string(tree.0.join("app/soplang.toml"))
        .unwrap()
        .contains("helper"));
    for args in [
        vec!["run", "app", "--ignored"],
        vec!["dhis", "app", "-O", "9"],
        vec!["dhis", "app", "--output"],
        vec!["dhis", "app", "-o", "a", "--output", "b"],
        vec!["add", "helper"],
        vec!["remove"],
        vec!["fmt", "app/src/main.sop", "--bad"],
        vec!["check", "app/src/main.sop", "--bad"],
        vec!["doc", "app/src/main.sop", "--qor"],
    ] {
        assert!(!cli(&tree, &args).status.success(), "{args:?}");
    }
    for command in [
        "fmt", "check", "doc", "run", "dhis", "build", "profile", "add", "remove", "update", "lsp",
        "debug", "test", "xog",
    ] {
        assert!(
            ok(cli(&tree, &[command, "--help"])).contains("Adeegsiga:"),
            "{command}"
        );
    }
}
