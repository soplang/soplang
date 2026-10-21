//! Package manifest discovery while running a source file.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

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
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
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

fn sop(dir: &Path, kayd: &Path, cache: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_soplang"));
    command
        .current_dir(dir)
        .env("SOPLANG_KAYD", kayd)
        .env("SOPLANG_CACHE", cache);
    command
}

fn assert_success(output: std::process::Output) -> String {
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    stdout
}

#[test]
fn a_registry_package_is_cached_pinned_and_updated() {
    let root = temp_dir("package_registry");
    let kayd = root.join("kayd");
    let cache = root.join("cache");
    let pkg = root.join("xisaab");
    let app = root.join("app");
    fs::create_dir_all(pkg.join("src")).unwrap();
    fs::create_dir_all(&app).unwrap();
    fs::write(
        pkg.join("soplang.toml"),
        "magac = \"xisaab\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\ngelid = \"src\"\n",
    )
    .unwrap();
    fs::write(
        pkg.join("src").join("xisaab.sop"),
        "soosaar wad\nhawl wad(a, b) { celi a + b }\n",
    )
    .unwrap();
    let published = sop(&pkg, &kayd, &cache).arg("--daabac").output().unwrap();
    assert!(assert_success(published).contains("0.1.0"), "publish 0.1.0");

    fs::write(
        app.join("soplang.toml"),
        "magac = \"app\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\n",
    )
    .unwrap();
    fs::write(
        app.join("main.sop"),
        "keen \"xisaab\" sida xisaab\ndoor add = xisaab.wad\nqor(add(1, 2))\n",
    )
    .unwrap();
    assert_success(
        sop(&app, &kayd, &cache)
            .args(["--ku-dar", "xisaab", "--qeexid", "kayd:^0.1.0"])
            .output()
            .unwrap(),
    );
    let lock = fs::read_to_string(app.join("soplang.lock")).unwrap();
    assert!(lock.contains("jid = \"kayd\""), "{lock}");
    assert!(lock.contains("sumad = \""), "{lock}");
    assert_eq!(
        assert_success(
            sop(&app, &kayd, &cache)
                .arg(app.join("main.sop"))
                .output()
                .unwrap()
        ),
        "3\n"
    );

    let published_file = kayd
        .join("xisaab")
        .join("0.1.0")
        .join("src")
        .join("xisaab.sop");
    let original = fs::read_to_string(&published_file).unwrap();
    fs::write(&published_file, "soosaar wad\nhawl wad(a, b) { celi 0 }\n").unwrap();
    let mismatch = sop(&app, &kayd, &cache)
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    assert!(
        !mismatch.status.success(),
        "changed bytes must fail the lock"
    );
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("soplang.lock"));
    fs::write(&published_file, original).unwrap();

    fs::write(
        pkg.join("soplang.toml"),
        "magac = \"xisaab\"\nnusxa = \"0.1.1\"\nsoplang = \"3\"\ngelid = \"src\"\n",
    )
    .unwrap();
    fs::write(
        pkg.join("src").join("xisaab.sop"),
        "soosaar wad\nhawl wad(a, b) { celi a + b + 10 }\n",
    )
    .unwrap();
    assert_success(sop(&pkg, &kayd, &cache).arg("--daabac").output().unwrap());
    assert_eq!(
        assert_success(
            sop(&app, &kayd, &cache)
                .arg(app.join("main.sop"))
                .output()
                .unwrap()
        ),
        "3\n"
    );
    assert_success(
        sop(&app, &kayd, &cache)
            .arg("--cusbooneysii")
            .output()
            .unwrap(),
    );
    let updated = fs::read_to_string(app.join("soplang.lock")).unwrap();
    assert!(updated.contains("nusxa = \"0.1.1\""), "{updated}");
    assert_eq!(
        assert_success(
            sop(&app, &kayd, &cache)
                .arg(app.join("main.sop"))
                .output()
                .unwrap()
        ),
        "13\n"
    );
    assert_success(
        sop(&app, &kayd, &cache)
            .args(["--ka-saar", "xisaab"])
            .output()
            .unwrap(),
    );
    let removed = sop(&app, &kayd, &cache)
        .arg(app.join("main.sop"))
        .output()
        .unwrap();
    assert!(
        !removed.status.success(),
        "{}",
        String::from_utf8_lossy(&removed.stderr)
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_http_archive_installs_into_the_cache() {
    let root = temp_dir("package_http");
    let kayd = root.join("kayd");
    let cache = root.join("cache");
    let pkg = root.join("xisaab");
    let app = root.join("app");
    fs::create_dir_all(pkg.join("src")).unwrap();
    fs::create_dir_all(&app).unwrap();
    fs::write(
        pkg.join("soplang.toml"),
        "magac = \"xisaab\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\ngelid = \"src\"\n",
    )
    .unwrap();
    fs::write(
        pkg.join("src").join("xisaab.sop"),
        "soosaar wad\nhawl wad(a, b) { celi a + b }\n",
    )
    .unwrap();
    assert_success(sop(&pkg, &kayd, &cache).arg("--daabac").output().unwrap());
    let body = fs::read(kayd.join("xisaab").join("xisaab-0.1.0.sopkg")).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let _ = listener.set_nonblocking(true);
        let started = Instant::now();
        let mut served = 0;
        while served < 4 && started.elapsed() < Duration::from_secs(20) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                    let mut buffer = [0u8; 2048];
                    let _ = stream.read(&mut buffer);
                    let header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(header.as_bytes());
                    let _ = stream.write_all(&body);
                    served += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20));
                }
                Err(_) => break,
            }
        }
    });
    fs::write(
        app.join("soplang.toml"),
        "magac = \"app\"\nnusxa = \"0.1.0\"\nsoplang = \"3\"\n",
    )
    .unwrap();
    fs::write(
        app.join("main.sop"),
        "keen \"xisaab\" sida xisaab\ndoor add = xisaab.wad\nqor(add(2, 3))\n",
    )
    .unwrap();
    let url = format!("http://127.0.0.1:{port}/xisaab-0.1.0.sopkg");
    assert_success(
        sop(&app, &kayd, &cache)
            .args(["--ku-dar", "xisaab", "--qeexid", &url])
            .output()
            .unwrap(),
    );
    assert_eq!(
        assert_success(
            sop(&app, &kayd, &cache)
                .arg(app.join("main.sop"))
                .output()
                .unwrap()
        ),
        "5\n"
    );
    let _ = fs::remove_dir_all(root);
}
