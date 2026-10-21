//! Git packages resolve from committed objects, pin one graph, and build offline.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Tree {
    root: PathBuf,
}
impl Tree {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "soplang_git_{label}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        Self { root }
    }
    fn app(&self, deps: &str) -> PathBuf {
        let app = self.root.join("app");
        fs::create_dir(&app).unwrap();
        fs::write(
            app.join("soplang.toml"),
            format!("magac='app'\nnusxa='0.1.0'\nsoplang='3'\n[ku-tiirsan]\n{deps}\n"),
        )
        .unwrap();
        fs::write(
            app.join("main.sop"),
            "keen \"xisaab\" sida m\nqor(m.value())\n",
        )
        .unwrap();
        app
    }
    fn command(&self, app: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_soplang"));
        command
            .current_dir(app)
            .env("SOPLANG_CACHE", self.root.join("cache"))
            .env("SOPLANG_KAYD", self.root.join("registry"))
            .env_remove("SOPLANG_OFFLINE");
        command
    }
    fn run(&self, app: &Path) -> String {
        success(
            self.command(app)
                .arg(app.join("main.sop"))
                .output()
                .unwrap(),
        )
    }
}
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_AUTHOR_NAME", "Soplang Tests")
        .env("GIT_AUTHOR_EMAIL", "tests@soplang.invalid")
        .env("GIT_COMMITTER_NAME", "Soplang Tests")
        .env("GIT_COMMITTER_EMAIL", "tests@soplang.invalid")
        .output()
        .unwrap();
    success(output)
}
fn success(output: std::process::Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
fn repo(root: &Path, name: &str, value: i64, deps: &str) -> PathBuf {
    let path = root.join(name);
    fs::create_dir(&path).unwrap();
    git(&path, &["init", "--initial-branch=main"]);
    fs::write(
        path.join("soplang.toml"),
        format!("magac='{name}'\nnusxa='1.0.0'\nsoplang='3'\n[ku-tiirsan]\n{deps}\n"),
    )
    .unwrap();
    fs::write(
        path.join("lib.sop"),
        format!("hawl value(){{celi {value}}}\n"),
    )
    .unwrap();
    commit(&path);
    path
}
fn commit(repo: &Path) -> String {
    git(repo, &["add", "--all"]);
    git(repo, &["commit", "-m", "Update package fixture"]);
    git(repo, &["rev-parse", "HEAD"]).trim().into()
}
fn url(repo: &Path) -> String {
    url::Url::from_file_path(repo).unwrap().to_string()
}
fn declaration(name: &str, repo: &Path, revision: &str) -> String {
    format!(
        "{name}={{git={},rev={}}}",
        toml::Value::String(url(repo)),
        toml::Value::String(revision.into())
    )
}
fn entries(app: &Path) -> Vec<toml::Value> {
    fs::read_to_string(app.join("soplang.lock"))
        .unwrap()
        .parse::<toml::Table>()
        .unwrap()["xirmo"]
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn git_packages_pin_committed_bytes_update_explicitly_and_build_offline() {
    let tree = Tree::new("pin with spaces");
    let origin = repo(&tree.root, "xisaab", 1, "");
    fs::write(origin.join(".gitignore"), "ignored.txt\n").unwrap();
    fs::create_dir(origin.join("data")).unwrap();
    fs::write(origin.join("data/af Soomaali.txt"), "xog").unwrap();
    let first = commit(&origin);
    fs::write(origin.join("lib.sop"), "hawl value(){celi 99}\n").unwrap(); // Dirty worktree is not a package source.
    let app = tree.app(&declaration("xisaab", &origin, "main"));
    assert_eq!(tree.run(&app), "1\n");
    let lock = entries(&app);
    assert_eq!(lock[0]["commit"].as_str(), Some(first.as_str()));
    let hash = lock[0]["sumad"].as_str().unwrap();
    assert_eq!(hash.len(), 64);
    assert_eq!(
        fs::read_to_string(
            tree.root
                .join("cache")
                .join(hash)
                .join("data/af Soomaali.txt")
        )
        .unwrap(),
        "xog"
    );
    let original_lock = fs::read(app.join("soplang.lock")).unwrap();
    fs::write(origin.join("lib.sop"), "hawl value(){celi 2}\n").unwrap();
    let second = commit(&origin);
    assert_eq!(tree.run(&app), "1\n");
    assert_eq!(fs::read(app.join("soplang.lock")).unwrap(), original_lock);
    let hidden = tree.root.join("origin-hidden");
    fs::rename(&origin, &hidden).unwrap();
    assert_eq!(
        success(
            tree.command(&app)
                .env("PATH", tree.root.join("no-git"))
                .arg("--offline")
                .arg(app.join("main.sop"))
                .output()
                .unwrap()
        ),
        "1\n"
    );
    for level in [0, 2] {
        let binary = app.join(format!("offline-{level}{}", std::env::consts::EXE_SUFFIX));
        success(
            tree.command(&app)
                .args(["--offline", "--build"])
                .arg(app.join("main.sop"))
                .arg("--opt-level")
                .arg(level.to_string())
                .arg("-o")
                .arg(&binary)
                .arg("-q")
                .output()
                .unwrap(),
        );
        let cached = tree.root.join("cache");
        let saved = tree.root.join("saved-cache");
        fs::rename(&cached, &saved).unwrap();
        fs::rename(app.join("main.sop"), app.join("saved-source.sop")).unwrap();
        assert_eq!(success(Command::new(binary).output().unwrap()), "1\n");
        fs::rename(app.join("saved-source.sop"), app.join("main.sop")).unwrap();
        fs::rename(saved, cached).unwrap();
    }
    fs::rename(hidden, &origin).unwrap();
    success(tree.command(&app).arg("--cusbooneysii").output().unwrap());
    assert_eq!(entries(&app)[0]["commit"].as_str(), Some(second.as_str()));
    assert_eq!(tree.run(&app), "2\n");
}

#[test]
fn corrupt_caches_and_cold_offline_builds_do_not_execute_unverified_source() {
    let tree = Tree::new("integrity");
    let origin = repo(&tree.root, "xisaab", 1, "");
    let app = tree.app(&declaration("xisaab", &origin, "HEAD"));
    let output = tree
        .command(&app)
        .arg("--offline")
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    assert!(!output.status.success() && output.stdout.is_empty());
    assert!(!app.join("soplang.lock").exists());
    assert_eq!(tree.run(&app), "1\n");
    let lock = entries(&app);
    let cached = tree
        .root
        .join("cache")
        .join(lock[0]["sumad"].as_str().unwrap());
    fs::write(cached.join("lib.sop"), "qor(999)").unwrap();
    let output = tree
        .command(&app)
        .arg("--offline")
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    assert!(!output.status.success() && output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("E057"));
    assert_eq!(tree.run(&app), "1\n");
    fs::create_dir(cached.join(".git")).unwrap();
    fs::write(cached.join(".git/injected.sop"), "qor(999)").unwrap();
    let output = tree
        .command(&app)
        .arg("--offline")
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    assert!(!output.status.success() && output.stdout.is_empty());
    assert_eq!(tree.run(&app), "1\n");
    assert!(!cached.join(".git").exists());
}

#[test]
fn transitive_imports_share_one_root_lock_and_reject_conflicting_commits() {
    let tree = Tree::new("graph");
    let base = repo(&tree.root, "base", 1, "");
    git(&base, &["tag", "v1"]);
    let child = repo(&tree.root, "xisaab", 0, &declaration("base", &base, "main"));
    fs::write(
        child.join("lib.sop"),
        "keen \"base\" sida base\nhawl value(){celi base.value()+10}\n",
    )
    .unwrap();
    fs::write(
        child.join("soplang.lock"),
        "not a valid lockfile for the consuming project",
    )
    .unwrap();
    commit(&child);
    let app = tree.app(&format!(
        "{}\n{}",
        declaration("xisaab", &child, "main"),
        declaration("base", &base, "v1")
    ));
    assert_eq!(tree.run(&app), "11\n");
    let lock = entries(&app);
    assert_eq!(lock.len(), 2);
    let base_lock = lock
        .iter()
        .find(|entry| entry["magac"].as_str() == Some("base"))
        .unwrap();
    assert_eq!(base_lock["codsiyo"].as_array().unwrap().len(), 2);
    let before = fs::read(app.join("soplang.lock")).unwrap();
    fs::write(base.join("lib.sop"), "hawl value(){celi 2}\n").unwrap();
    commit(&base);
    assert_eq!(tree.run(&app), "11\n");
    let update = tree.command(&app).arg("--cusbooneysii").output().unwrap();
    assert!(!update.status.success());
    assert_eq!(fs::read(app.join("soplang.lock")).unwrap(), before);
    let manifest = fs::read_to_string(app.join("soplang.toml"))
        .unwrap()
        .replace("rev=\"v1\"", "rev=\"main\"");
    fs::write(app.join("soplang.toml"), manifest).unwrap();
    success(tree.command(&app).arg("--cusbooneysii").output().unwrap());
    assert_eq!(tree.run(&app), "12\n");
}

#[test]
fn changed_lock_sources_are_rejected_before_trying_another_remote() {
    let tree = Tree::new("source");
    let origin = repo(&tree.root, "xisaab", 1, "");
    let app = tree.app(&declaration("xisaab", &origin, "main"));
    assert_eq!(tree.run(&app), "1\n");
    fs::write(app.join("soplang.toml"),format!("magac='app'\nnusxa='0.1.0'\nsoplang='3'\n[ku-tiirsan]\nxisaab={{git='file:///missing-repository',rev='main'}}\n")).unwrap();
    let output = tree
        .command(&app)
        .env("PATH", tree.root.join("no-git"))
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success() && output.stdout.is_empty());
    assert!(
        error.contains("soplang.lock") && !error.contains("Git lama bilaabin"),
        "{error}"
    );
}

#[test]
fn concurrent_installers_publish_one_complete_cache_and_lock() {
    let tree = Tree::new("parallel");
    let origin = repo(&tree.root, "xisaab", 1, "");
    let app = tree.app(&declaration("xisaab", &origin, "main"));
    let mut children = Vec::new();
    for _ in 0..6 {
        children.push(
            tree.command(&app)
                .arg(app.join("main.sop"))
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap(),
        );
    }
    for child in children {
        assert_eq!(success(child.wait_with_output().unwrap()), "1\n");
    }
    assert_eq!(entries(&app).len(), 1);
    for entry in fs::read_dir(tree.root.join("cache")).unwrap() {
        let name = entry.unwrap().file_name();
        let name = name.to_string_lossy();
        assert!(
            !name.starts_with(".git-fetch-") && !name.starts_with(".tree-"),
            "{name}"
        );
    }
    assert_eq!(tree.run(&app), "1\n");
}

#[test]
fn names_cycles_and_escaped_package_files_are_rejected() {
    let tree = Tree::new("invalid");
    let origin = repo(&tree.root, "wrong", 1, "");
    let app = tree.app(&declaration("xisaab", &origin, "main"));
    let output = tree
        .command(&app)
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    assert!(!output.status.success() && output.stdout.is_empty());
    assert!(!app.join("soplang.lock").exists());
    fs::remove_dir_all(&app).unwrap();
    let a = repo(&tree.root, "a", 1, "");
    let b = repo(&tree.root, "b", 2, &declaration("a", &a, "main"));
    let text = fs::read_to_string(a.join("soplang.toml")).unwrap() + &declaration("b", &b, "main");
    fs::write(a.join("soplang.toml"), text).unwrap();
    commit(&a);
    let app = tree.app(&declaration("a", &a, "main"));
    let output = tree
        .command(&app)
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success() && error.contains("wareeg"),
        "{error}"
    );
    assert!(!app.join("soplang.lock").exists());
    fs::remove_dir_all(&app).unwrap();
    let escape = repo(&tree.root, "xisaab", 1, "");
    fs::write(
        escape.join("lib.sop"),
        "keen \"../outside.sop\"\nhawl value(){celi 1}\n",
    )
    .unwrap();
    commit(&escape);
    let app = tree.app(&declaration("xisaab", &escape, "main"));
    // The outside file exists next to the materialized content trees.
    fs::create_dir_all(tree.root.join("cache")).unwrap();
    fs::write(tree.root.join("cache/outside.sop"), "qor(999)\n").unwrap();
    let output = tree
        .command(&app)
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success() && output.stdout.is_empty() && error.contains("ka baxsan"),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn git_timeouts_cover_transport_children_and_inherited_pipes() {
    use std::os::unix::fs::PermissionsExt;
    let tree = Tree::new("timeout");
    let origin = repo(&tree.root, "xisaab", 1, "");
    let app = tree.app(&declaration("xisaab", &origin, "main"));
    let bin = tree.root.join("bin");
    fs::create_dir(&bin).unwrap();
    let fake = bin.join("git");
    for script in [
        "#!/bin/sh\n/bin/sleep 5 &\nwait\n",
        "#!/bin/sh\n/bin/sleep 5 &\nexit 0\n",
    ] {
        fs::write(&fake, script).unwrap();
        fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
        let start = std::time::Instant::now();
        let output = tree
            .command(&app)
            .env("PATH", &bin)
            .env("SOPLANG_GIT_TIMEOUT_MS", "80")
            .arg(app.join("main.sop"))
            .output()
            .unwrap();
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && error.contains("Waqtigii Git"),
            "{error}"
        );
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
    }
}

#[test]
fn package_commands_support_annotated_tags_and_full_commit_selectors() {
    let tree = Tree::new("selectors");
    let origin = repo(&tree.root, "xisaab", 7, "");
    let first = git(&origin, &["rev-parse", "HEAD"]).trim().to_string();
    git(&origin, &["tag", "-a", "v1", "-m", "Tag package fixture"]);
    let app = tree.app("");
    let tagged = format!("git+{}#v1", url(&origin));
    success(
        tree.command(&app)
            .args(["--ku-dar", "xisaab", "--qeexid", &tagged])
            .output()
            .unwrap(),
    );
    assert_eq!(entries(&app)[0]["commit"].as_str(), Some(first.as_str()));
    assert_eq!(tree.run(&app), "7\n");
    success(
        tree.command(&app)
            .args(["--ka-saar", "xisaab"])
            .output()
            .unwrap(),
    );
    assert!(!fs::read_to_string(app.join("soplang.toml"))
        .unwrap()
        .contains("xisaab"));
    let pinned = format!("git+{}#{first}", url(&origin));
    success(
        tree.command(&app)
            .args(["--ku-dar", "xisaab", "--qeexid", &pinned])
            .output()
            .unwrap(),
    );
    assert_eq!(tree.run(&app), "7\n");
}

#[cfg(unix)]
#[test]
fn git_symlinks_are_rejected_as_data_before_materialization() {
    let tree = Tree::new("symlink");
    let origin = repo(&tree.root, "xisaab", 1, "");
    std::os::unix::fs::symlink("../../outside", origin.join("link")).unwrap();
    commit(&origin);
    let app = tree.app(&declaration("xisaab", &origin, "main"));
    let output = tree
        .command(&app)
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    assert!(!output.status.success() && output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("faylal caadi ah"));
    assert!(!app.join("soplang.lock").exists());
    assert!(fs::read_dir(tree.root.join("cache"))
        .unwrap()
        .all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".git-fetch-")));
}

#[test]
fn sha256_git_repositories_keep_full_commit_identifiers() {
    let tree = Tree::new("sha256");
    let origin = tree.root.join("xisaab");
    fs::create_dir(&origin).unwrap();
    git(
        &origin,
        &["init", "--initial-branch=main", "--object-format=sha256"],
    );
    fs::write(
        origin.join("soplang.toml"),
        "magac='xisaab'\nnusxa='1.0.0'\nsoplang='3'\n",
    )
    .unwrap();
    fs::write(origin.join("lib.sop"), "hawl value(){celi 256}\n").unwrap();
    let oid = commit(&origin);
    assert_eq!(oid.len(), 64);
    let app = tree.app(&declaration("xisaab", &origin, "main"));
    assert_eq!(tree.run(&app), "256\n");
    assert_eq!(entries(&app)[0]["commit"].as_str(), Some(oid.as_str()));
}

#[test]
fn repl_projects_keep_one_resolution_until_the_session_restarts() {
    use std::io::{Read, Write};
    if let Some(path) = std::env::var_os("SOPLANG_REPL_PROJECT_FIXTURE") {
        let path = PathBuf::from(path);
        let mut session = soplang::ReplSession::new(false).unwrap();
        session
            .eval("keen \"xisaab\" sida first", Some(&path))
            .unwrap();
        println!("snapshot-ready");
        std::io::stdout().flush().unwrap();
        let mut signal = String::new();
        std::io::stdin().read_line(&mut signal).unwrap();
        session
            .eval(
                "keen \"xisaab\" sida second\ndoor result=second.value()",
                None,
            )
            .unwrap();
        assert_eq!(session.get_global("result"), Some(soplang::Value::Int(1)));
        return;
    }
    let tree = Tree::new("repl");
    let origin = repo(&tree.root, "xisaab", 1, "");
    let app = tree.app(&declaration("xisaab", &origin, "main"));
    assert_eq!(tree.run(&app), "1\n");
    let source = format!(
        "keen \"{}\" sida application",
        app.join("main.sop").to_string_lossy().replace('\\', "/")
    );
    assert_eq!(
        success(tree.command(&app).args(["-c", &source]).output().unwrap()),
        "1\n"
    );
    fs::write(origin.join("lib.sop"), "hawl value(){celi 2}\n").unwrap();
    commit(&origin);
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "repl_projects_keep_one_resolution_until_the_session_restarts",
            "--nocapture",
        ])
        .current_dir(&app)
        .env("SOPLANG_REPL_PROJECT_FIXTURE", app.join("main.sop"))
        .env("SOPLANG_CACHE", tree.root.join("cache"))
        .env_remove("SOPLANG_OFFLINE")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut prefix = Vec::new();
    let mut byte = [0u8; 1];
    while !prefix.ends_with(b"snapshot-ready\n") {
        stdout.read_exact(&mut byte).unwrap();
        prefix.push(byte[0]);
    }
    success(tree.command(&app).arg("--cusbooneysii").output().unwrap());
    let mut input = child.stdin.take().unwrap();
    input.write_all(b"continue\n").unwrap();
    drop(input);
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{rest}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(tree.run(&app), "2\n");
}
