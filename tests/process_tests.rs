//! Program arguments and environment variables (3.3 stage).

use std::fs;
use std::process::Command;

fn run(args: &[&str], env: &[(&str, &str)]) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let mut command = Command::new(exe);
    command.args(args);
    for (name, value) in env {
        command.env(name, value);
    }
    let out = command.output().expect("run soplang");
    assert!(
        out.status.success(),
        "program failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn command_arguments_and_environment_round_trip() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door args = amarro()
hubi(dherer(args) == 2 && args[0] == "one" && args[1] == "--two", "args")
hubi(deegaan("SOPLANG_PROCESS_PARENT") == "parent", "parent env")
hubi(deegaan("SOPLANG_PROCESS_MISSING") == maran, "missing")
deegaan_qor("SOPLANG_PROCESS_CHILD", "haa")
hubi(deegaan("SOPLANG_PROCESS_CHILD") == "haa", "set")
deegaan_tirtir("SOPLANG_PROCESS_CHILD")
hubi(deegaan("SOPLANG_PROCESS_CHILD") == maran, "removed")
qor("process ok")
"#;
    assert_eq!(
        run(
            &["-c", source, "one", "--two"],
            &[("SOPLANG_PROCESS_PARENT", "parent")]
        ),
        "process ok\n"
    );
}

#[test]
fn file_arguments_exclude_the_program_name() {
    let dir = std::env::temp_dir().join(format!(
        "soplang_args_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("args.sop");
    fs::write(
        &file,
        "hawl hubi(v, name) { haddii (!v) { tuur name } }\ndoor args = amarro()\nhubi(dherer(args) == 1 && args[0] == \"file-arg\", \"file\")\nqor(\"file args ok\")\n",
    )
    .unwrap();
    assert_eq!(
        run(&[file.to_str().unwrap(), "file-arg"], &[]),
        "file args ok\n"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn invalid_environment_names_are_catchable() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door failed = been
tijaabi { deegaan_qor("", "x") } qabo (e) { failed = run }
hubi(failed, "name")
qor("process errors ok")
"#;
    assert_eq!(run(&["-c", source], &[]), "process errors ok\n");
}

#[test]
#[cfg(unix)]
fn child_process_captures_output_and_exit_code() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door ok = fuli("/bin/echo", ["Salaan"])
hubi(ok.guul && ok.koodh == 0 && ok.soosaar == e"Salaan\n" && ok.qalad == "", "echo")
door failed = fuli("/usr/bin/false")
hubi(!failed.guul && failed.koodh == 1 && failed.soosaar == "", "false")
door missing = been
tijaabi { fuli("/bin/soplang-missing-command") } qabo (e) { missing = run }
hubi(missing, "missing")
qor("child ok")
"#;
    assert_eq!(run(&["-c", source], &[]), "child ok\n");
}

#[test]
#[cfg(windows)]
fn child_process_captures_output_and_exit_code() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door ok = fuli("cmd.exe", ["/C", "echo Salaan"])
hubi(ok.guul && ok.koodh == 0 && ok.soosaar != "" && ok.qalad == "", "echo")
door failed = fuli("cmd.exe", ["/C", "exit /B 1"])
hubi(!failed.guul && failed.koodh == 1 && failed.soosaar == "", "false")
door missing = been
tijaabi { fuli("soplang-missing-command") } qabo (e) { missing = run }
hubi(missing, "missing")
qor("child ok")
"#;
    assert_eq!(run(&["-c", source], &[]), "child ok\n");
}
