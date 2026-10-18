//! User-visible Somali output and preserved technical/source context.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use soplang::error::{file_read_error, io_error, tool_start_error};

struct Work(PathBuf);
impl Work {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "soplang Somali {} {} {}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Work {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_soplang"));
    command
        .env("NO_COLOR", "1")
        .env_remove("SOPLANG_ERROR_DETAILS");
    command
}
fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone())
        .unwrap()
        .replace("\r\n", "\n")
}
fn somali_failure(output: &Output, code: i32, message: &str) {
    let text = stderr(output);
    assert_eq!(output.status.code(), Some(code), "{text}");
    assert!(output.stdout.is_empty(), "unexpected stdout: {output:?}");
    assert!(text.contains(message), "{text}");
    for english in [
        "Usage:",
        "error: unexpected",
        "invalid digit",
        "out of range",
        "No such file",
        "os error",
        "stream did not",
        "thread '",
        "panicked",
        "safka 0",
        "tiirka 0",
        "Faahfaahin farsamo:",
    ] {
        assert!(!text.contains(english), "{text}");
    }
}

#[test]
fn help_and_argument_errors_keep_their_streams_and_statuses() {
    for flag in ["--help", "-h"] {
        let output = cli().arg(flag).output().unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let help = String::from_utf8(output.stdout).unwrap();
        for expected in [
            "Af-Soomaaliga",
            "Adeegsiga:",
            "Faylka:",
            "Xulashooyinka:",
            "[FAYLKA]",
            "<HEERKA>",
            "Muuji caawimada.",
            "Muuji nooca Soplang.",
            "haddii aan la sheegin: 2",
        ] {
            assert!(help.contains(expected), "{help}");
        }
        for english in [
            "Usage",
            "Arguments",
            "Options",
            "[default:",
            "Print help",
            "Print version",
            "FILENAME",
            "OUTPUT",
            "With no arguments",
        ] {
            assert!(!help.contains(english), "{help}");
        }
    }
    for flag in ["--version", "-V"] {
        let output = cli().arg(flag).output().unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            format!("soplang {}", env!("CARGO_PKG_VERSION"))
        );
    }
    for (arguments, message) in [
        (vec!["--not-a-soplang-option"], "lama aqoonsan"),
        (vec!["--file"], "waxay u baahan tahay qiime"),
        (vec!["--opt-level", "abc"], "ma saxna"),
        (vec!["--opt-level", "99999999999999"], "ma saxna"),
        (vec!["--example", "abc"], "ma saxna"),
        (vec!["--quiet", "--quiet"], "waa la soo celiyey"),
    ] {
        somali_failure(&cli().args(arguments).output().unwrap(), 2, message);
    }
}

#[test]
fn file_failures_and_repl_loads_use_contextual_somali() {
    let work = Work::new();
    let missing = work.0.join("fayl maqan.sop");
    for args in [
        vec![missing.as_os_str()],
        vec!["--file".as_ref(), missing.as_os_str()],
        vec!["--build".as_ref(), "--quiet".as_ref(), missing.as_os_str()],
    ] {
        somali_failure(&cli().args(args).output().unwrap(), 1, "lama helin");
    }
    let invalid = work.0.join("qoraal aan sax ahayn-é.sop");
    fs::write(&invalid, [0xff, 0xfe]).unwrap();
    somali_failure(
        &cli().arg(&invalid).output().unwrap(),
        1,
        "ma aha qoraal UTF-8 ah oo sax ah",
    );
    let mut child = cli()
        .env("HOME", &work.0)
        .env("USERPROFILE", &work.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // /akhrifayl currently tokenizes on whitespace; use a simple guaranteed-missing path.
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"/akhrifayl soplang-audit-missing.sop\n1 + 2\n/bixi\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(stderr(&output).contains("Faylka 'soplang-audit-missing.sop' lama helin."));
    assert!(!stderr(&output).contains("No such file"));
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .replace("soplang> ", "")
        .lines()
        .any(|line| line.trim() == "3"));
}

#[test]
fn io_causes_and_internal_runtime_tags_are_explained_without_raw_text() {
    use std::io::{Error, ErrorKind};
    for (kind, expected) in [
        (ErrorKind::PermissionDenied, "Lama oggola"),
        (ErrorKind::IsADirectory, "waa gal"),
        (ErrorKind::InvalidData, "UTF-8"),
    ] {
        let error = file_read_error(
            Path::new("magac.sop"),
            &Error::new(kind, "English fixture reason"),
        );
        assert!(error.message().contains(expected));
        assert!(!error.to_string().contains("English"));
        assert_eq!(
            error.meta().technical_details.as_deref(),
            Some("English fixture reason")
        );
    }
    for (kind, expected) in [
        (ErrorKind::StorageFull, "buuxdaa"),
        (ErrorKind::ReadOnlyFilesystem, "akhris"),
        (ErrorKind::Other, "lama aqoonsan"),
    ] {
        let error = io_error(
            "Faylka lama qori karin.",
            &Error::new(kind, "English fixture reason"),
        );
        assert!(error.message().contains(expected));
    }
    let denied = tool_start_error(
        "clang",
        &Error::new(ErrorKind::PermissionDenied, "English fixture reason"),
    );
    assert!(!denied.message().contains("Rakib"));
    let error = soplang::runtime::abi::soplang_to_value(soplang::runtime::abi::SoplangValue {
        tag: 255,
        _pad: [0; 7],
        payload: 0,
    })
    .unwrap_err();
    assert_eq!(error.message(), "Summadda qiimaha ma saxna: 255.");
    let backend = soplang::backend::cranelift::CraneliftBackend::new().unwrap();
    assert_eq!(
        backend.run_main().unwrap_err().message(),
        "Hawsha 'main' weli lama ururin."
    );
}

#[test]
fn hir_localizes_the_wrapper_and_preserves_instruction_notation() {
    let work = Work::new();
    let file = work.0.join("tusaale-é.sop");
    fs::write(&file, "qor(1)\n").unwrap();
    let output = cli().arg("--hir").arg(&file).output().unwrap();
    assert!(output.status.success());
    let hir = String::from_utf8(output.stdout).unwrap();
    assert!(hir.contains("--- qaybta guud ---"));
    assert!(!hir.contains("--- top_level ---"));
    assert!(hir.contains("call") && hir.contains("location 1:1"));
}

fn invalid_input(mut command: Command, details: bool) -> Output {
    if details {
        command.env("SOPLANG_ERROR_DETAILS", "1");
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&[0xff, b'\n'])
        .unwrap();
    child.wait_with_output().unwrap()
}
fn check_input_errors(mut command: impl FnMut() -> Command) {
    for details in [false, true] {
        let output = invalid_input(command(), details);
        assert_eq!(output.status.code(), Some(1));
        let text = stderr(&output);
        assert!(
            text.contains("Xogta la geliyey ma aha qoraal UTF-8 ah oo sax ah."),
            "{text}"
        );
        assert!(
            text.contains("safka 1, tiirka 1") && text.contains("│   ^^^^^"),
            "{text}"
        );
        assert_eq!(text.contains("Faahfaahin farsamo:"), details, "{text}");
        assert_eq!(
            text.contains("stream did not contain valid UTF-8"),
            details,
            "{text}"
        );
    }
}
#[test]
fn input_errors_retain_spans_and_only_show_details_when_requested() {
    check_input_errors(|| {
        let mut command = cli();
        command.args(["-c", "gelin()"]);
        command
    });
}

#[test]
fn test_aot_localized_input_and_caught_errors_match_jit() {
    let work = Work::new();
    let file = work.0.join("gelin.sop");
    let binary = work
        .0
        .join("gelin")
        .with_extension(std::env::consts::EXE_EXTENSION);
    fs::write(&file, "gelin()\n").unwrap();
    let build = cli()
        .args(["--build", "--quiet"])
        .arg(&file)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(build.status.success(), "{}", stderr(&build));
    check_input_errors(|| {
        let mut command = Command::new(&binary);
        command
            .env("NO_COLOR", "1")
            .env_remove("SOPLANG_ERROR_DETAILS");
        command
    });
    fs::write(&file, "tijaabi { gelin() } qabo (e) { qor(e) }\n").unwrap();
    let build = cli()
        .args(["--build", "--quiet"])
        .arg(&file)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(build.status.success(), "{}", stderr(&build));
    for details in [false, true] {
        let jit = invalid_input(
            {
                let mut command = cli();
                command.arg(&file);
                command
            },
            details,
        );
        let native = invalid_input(
            {
                let mut command = Command::new(&binary);
                command
                    .env("NO_COLOR", "1")
                    .env_remove("SOPLANG_ERROR_DETAILS");
                command
            },
            details,
        );
        assert!(jit.status.success() && native.status.success());
        assert!(jit.stderr.is_empty() && native.stderr.is_empty());
        assert_eq!(jit.stdout, native.stdout);
        assert_eq!(
            String::from_utf8(jit.stdout).unwrap(),
            "Xogta la geliyey ma aha qoraal UTF-8 ah oo sax ah.\n"
        );
    }
}

#[test]
fn runtime_metadata_errors_have_no_fake_source_location() {
    let work = Work::new();
    let file = work.0.join("hello.sop");
    fs::write(&file, "qor(1)\n").unwrap();
    let mut command = cli();
    command
        .args(["--build", "--quiet"])
        .arg(&file)
        .arg("-o")
        .arg(work.0.join("out"))
        .env("SOPLANG_RUNTIME_DIR", &work.0);
    somali_failure(&command.output().unwrap(), 1, "Xogta nooca Rust");
    let output = command.env("SOPLANG_ERROR_DETAILS", "1").output().unwrap();
    assert!(stderr(&output).contains("Faahfaahin farsamo:"));
}

#[cfg(unix)]
#[test]
fn invalid_argument_encoding_is_reported_in_somali() {
    use std::os::unix::ffi::OsStringExt;
    let invalid = std::ffi::OsString::from_vec(vec![0xff]);
    somali_failure(
        &cli().arg("--command").arg(invalid).output().unwrap(),
        2,
        "UTF-8",
    );
}

#[test]
fn tool_failures_preserve_stderr_only_in_requested_details() {
    let work = Work::new();
    let source = work.0.join("hello.sop");
    fs::write(&source, "qor(1)\n").unwrap();
    let tools = work.0.join("tools");
    fs::create_dir(&tools).unwrap();
    let fixture = work.0.join("tool.rs");
    fs::write(&fixture, r#"
fn main() {
    let name = std::env::current_exe().unwrap();
    if name.file_stem().unwrap() == "clang" && std::env::var_os("SOPLANG_FIXTURE_CLANG_OK").is_some() {
        return;
    }
    eprintln!("English tool fixture reason");
    std::process::exit(42);
}
"#).unwrap();
    let clang = tools
        .join("clang")
        .with_extension(std::env::consts::EXE_EXTENSION);
    let compiled = Command::new("rustc")
        .arg("--crate-name")
        .arg("soplang_tool_fixture")
        .arg(&fixture)
        .arg("-o")
        .arg(&clang)
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{}", stderr(&compiled));
    let run = |details, clang_ok| {
        let mut command = cli();
        command
            .args(["--build", "--quiet"])
            .arg(&source)
            .arg("-o")
            .arg(work.0.join("out"))
            .env("PATH", &tools)
            .env_remove("SOPLANG_RUNTIME_DIR");
        if details {
            command.env("SOPLANG_ERROR_DETAILS", "1");
        }
        if clang_ok {
            command.env("SOPLANG_FIXTURE_CLANG_OK", "1");
        }
        command.output().unwrap()
    };
    for details in [false, true] {
        let output = run(details, false);
        assert_eq!(output.status.code(), Some(1));
        let text = stderr(&output);
        assert!(
            text.contains("clang ma dhammaystiri karin ururinta LLVM IR."),
            "{text}"
        );
        assert_eq!(text.contains("English tool fixture reason"), details);
        assert_eq!(text.contains("42"), details);
        assert!(!text.contains("safka 0"));
    }
    fs::copy(
        &clang,
        tools
            .join("rustc")
            .with_extension(std::env::consts::EXE_EXTENSION),
    )
    .unwrap();
    for details in [false, true] {
        let output = run(details, true);
        assert_eq!(output.status.code(), Some(1));
        let text = stderr(&output);
        assert!(
            text.contains("rustc ma dhammaystiri karin isku xidhka barnaamijka."),
            "{text}"
        );
        assert_eq!(text.contains("English tool fixture reason"), details);
    }
}

#[test]
fn broken_output_is_a_runtime_diagnostic_instead_of_a_panic() {
    let mut child = cli()
        .args(["-c", "qor(gelin())"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    child.stdin.take().unwrap().write_all(b"Salaan\n").unwrap();
    let output = child.wait_with_output().unwrap();
    somali_failure(&output, 1, "Waxsoosaarka lama qori karin.");
}
