//! Human-reviewed error examples: status, Somali message, original file,
//! one-based line/column, source text, and exact caret spacing in both backends.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Case {
    phase: &'static str,
    path: PathBuf,
    line: usize,
    col: usize,
    message: &'static str,
}

fn cases() -> Vec<Case> {
    include_str!("../examples/errors/cases.tsv")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 5, "invalid diagnostic manifest: {line}");
            Case {
                phase: fields[0],
                path: Path::new("examples/errors").join(fields[1]),
                line: fields[2].parse().unwrap(),
                col: fields[3].parse().unwrap(),
                message: fields[4],
            }
        })
        .collect()
}

fn check(case: &Case, result: Output) {
    assert_eq!(
        result.status.code(),
        Some(1),
        "{}: {}",
        case.path.display(),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        result.stdout.is_empty(),
        "{} unexpectedly wrote stdout",
        case.path.display()
    );
    let stderr = String::from_utf8(result.stderr)
        .unwrap()
        .replace("\r\n", "\n");
    // File separators depend on the host; source text and caret spacing do not.
    let mut in_trace = false;
    let stderr = stderr
        .split_inclusive('\n')
        .map(|line| {
            let frame = in_trace && line.starts_with("     ");
            in_trace = frame || line.trim_end() == "Raadka wicitaannada:";
            if line.starts_with("     --> ") || frame {
                line.replace('\\', "/")
            } else {
                line.to_owned()
            }
        })
        .collect::<String>();
    assert!(stderr.starts_with("Khalad "), "{stderr}");
    assert!(
        stderr.contains(&format!("safka {}, tiirka {}", case.line, case.col)),
        "{}: {stderr}",
        case.path.display()
    );
    assert!(
        stderr.contains(case.message),
        "{}: {stderr}",
        case.path.display()
    );
    assert!(
        stderr.contains("     --> ") && stderr.contains("     │   ") && stderr.contains('^'),
        "{stderr}"
    );
    let expected =
        fs::read_to_string(case.path.with_extension("stderr")).expect("diagnostic snapshot");
    // Ignore only the final blank line; source and caret whitespace remain exact.
    assert_eq!(
        stderr.trim_end(),
        expected.replace("\r\n", "\n").trim_end(),
        "{} diagnostic",
        case.path.display()
    );
}

#[test]
fn test_error_examples_jit() {
    for case in cases() {
        let mut command = Command::new(env!("CARGO_BIN_EXE_soplang"));
        command.arg("--no-color");
        if case.phase == "strict" {
            command.arg("--strict");
        }
        check(&case, command.arg(&case.path).output().unwrap());
    }
}

#[test]
fn test_aot_error_examples() {
    let dir = std::env::temp_dir().join(format!("soplang_diagnostics_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    for case in cases() {
        let binary = dir
            .join(case.path.file_stem().unwrap())
            .with_extension(std::env::consts::EXE_EXTENSION);
        let mut command = Command::new(env!("CARGO_BIN_EXE_soplang"));
        command.args(["--no-color", "--build", "--quiet"]);
        if case.phase == "strict" {
            command.arg("--strict");
        }
        let build = command
            .arg(&case.path)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        if case.phase == "runtime" {
            assert!(
                build.status.success(),
                "{} failed to build: {}",
                case.path.display(),
                String::from_utf8_lossy(&build.stderr)
            );
            check(
                &case,
                Command::new(&binary).env("NO_COLOR", "1").output().unwrap(),
            );
            fs::remove_file(binary).unwrap();
        } else {
            check(&case, build);
            assert!(!binary.exists(), "invalid program produced an executable");
        }
    }
    fs::remove_dir(dir).unwrap();
}

#[test]
fn test_command_expression_keeps_original_columns() {
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .args(["--no-color", "-c", "10 / 0"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("<command>:1:4"), "{stderr}");
    assert!(
        stderr.contains("   1 │   10 / 0\n     │      ^"),
        "{stderr}"
    );
    assert!(
        !stderr.contains("qor("),
        "echo wrapper changed the displayed source"
    );
}

#[test]
fn test_repl_compile_error_keeps_original_columns() {
    let mut session = soplang::ReplSession::new(false).unwrap();
    let error = session.eval("door n =\n", None).unwrap_err();
    assert_eq!(error.location(), (2, 1));
    let formatted = soplang::format_error_with_source(&error, None);
    assert!(formatted.contains("   2 │   \n     │   ^"), "{formatted}");
}

#[test]
fn test_error_manifest_covers_every_case() {
    let mut declared: Vec<_> = cases().into_iter().map(|case| case.path).collect();
    let mut actual: Vec<_> = fs::read_dir("examples/errors")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sop"))
        .collect();
    declared.sort();
    actual.sort();
    assert_eq!(
        declared, actual,
        "every error example must have a manifest entry"
    );
}

#[test]
fn test_caret_uses_display_width_for_tabs_and_combining_characters() {
    for (source, col, end_col, rendered, padding, width) in [
        ("\t中é🙂 bad", 6, 8, "    中é🙂 bad", 10, 3),
        ("e\u{301} + x", 6, 6, "e\u{301} + x", 4, 1),
        ("qor(", 5, 5, "qor(", 4, 1),
    ] {
        let span = soplang::error::SourceSpan {
            file: None,
            line: 1,
            col,
            end_col,
            source_line: source.into(),
        };
        let error = span.attach(soplang::error::runtime_error("Tusaale", 0, 0));
        let formatted = soplang::format_error_with_source(&error, None);
        assert!(
            formatted.contains(&format!("   1 │   {rendered}\n")),
            "{formatted}"
        );
        assert!(
            formatted.contains(&format!(
                "     │   {}{}\n",
                " ".repeat(padding),
                "^".repeat(width)
            )),
            "{formatted}"
        );
    }
}

#[test]
fn test_command_expression_parser_error_points_at_original_eof() {
    let result = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .args(["--no-color", "-c", "1 +"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(stderr.contains("<command>:1:4"), "{stderr}");
    assert!(stderr.contains("   1 │   1 +\n     │      ^"), "{stderr}");
}

#[test]
fn test_aot_runtime_error_survives_source_removal() {
    let dir = std::env::temp_dir().join(format!("soplang source removal {}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let main = dir.join("main.sop");
    let helper = dir.join("helper.sop");
    let binary = dir
        .join(format!("without_source_{}", std::process::id()))
        .with_extension(std::env::consts::EXE_EXTENSION);
    fs::write(&main, "keen \"helper.sop\"\nkhalad()\n").unwrap();
    fs::write(&helper, "hawl khalad() {\n    celi 8 / 0\n}\n").unwrap();
    let build = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .args(["--no-color", "--build", "--quiet"])
        .arg(&main)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    fs::remove_file(main).unwrap();
    fs::remove_file(&helper).unwrap();
    let result = Command::new(&binary).env("NO_COLOR", "1").output().unwrap();
    assert_eq!(result.status.code(), Some(1));
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains(&format!("{}:2:12", helper.display())),
        "{stderr}"
    );
    assert!(
        stderr.contains("   2 │       celi 8 / 0\n     │              ^"),
        "{stderr}"
    );
    fs::remove_file(binary).unwrap();
    fs::remove_dir(dir).unwrap();
}

#[test]
fn test_large_line_number_keeps_caret_gutter_aligned() {
    let span = soplang::error::SourceSpan {
        file: None,
        line: 10000,
        col: 3,
        end_col: 5,
        source_line: "  bad".into(),
    };
    let error = span.attach(soplang::error::runtime_error("Tusaale", 0, 0));
    let formatted = soplang::format_error_with_source(&error, None);
    assert!(
        formatted.contains("10000 │     bad\n      │     ^^^\n"),
        "{formatted}"
    );
}

#[test]
fn test_argument_errors_use_the_called_spelling() {
    for (source, expected, unwanted) in [
        ("qor(duub())", "duub()", "dhammeystir()"),
        ("qor(yar())", "yar()", "ugu_yar()"),
        ("qor(weyn())", "weyn()", "ugu_weyn()"),
        ("qor(jibbaar(2))", "jibbaar()", "awood()"),
        ("door t = []; t.hore(1)", "hore()", "koowaad()"),
        ("door t = []; t.dambe(1)", "dambe()", "ugu_dambeeya()"),
        ("door t = []; t.raadi()", "raadi()", "muuji()"),
        ("door w = {}; w.furayaal(1)", "furayaal()", "fure()"),
        ("door w = {}; w.qiimayaal(1)", "qiimayaal()", "qiime()"),
        (
            "door w = {}; w.lammaanayaal(1)",
            "lammaanayaal()",
            "lamaane()",
        ),
        ("door w = {}; w.midee()", "midee()", "kudar()"),
        ("door q = \"ab\"; q.xiriiri()", "xiriiri()", "kudar()"),
        (
            "door q = \"ab\"; q.weynee(1)",
            "weynee()",
            "xarfaha_waaweyn()",
        ),
        ("door q = \"ab\"; q.yaree(1)", "yaree()", "xarfaha_yaryar()"),
        ("door q = \"ab\"; q.kuceli(-1)", "kuceli()", "ku_celi()"),
        ("door q = \"ab\"; q.ku_celi(-1)", "ku_celi()", "kuceli()"),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_soplang"))
            .args(["--no-color", "-c", source])
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        let stderr = String::from_utf8(result.stderr).unwrap();
        let header = stderr.lines().next().unwrap();
        assert!(
            header.contains(expected) && !header.contains(unwanted),
            "{source}: {stderr}"
        );
    }
}
