//! Source formatting must preserve executable language behavior and comments.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use soplang::frontend::{formatter::format_source, parse_source};

fn sources(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            sources(&entry.path(), files);
        } else if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "sop")
        {
            files.push(entry.path());
        }
    }
}

#[test]
fn formatting_the_language_corpus_preserves_valid_syntax_and_is_stable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for directory in ["examples", "tests/fixtures", "src/stdlib"] {
        sources(&root.join(directory), &mut files);
    }
    let mut checked = 0;
    for path in files {
        let original = fs::read_to_string(&path).unwrap();
        if parse_source(&original, Some(&path)).is_err() {
            continue;
        }
        let formatted = format_source(&original, &path)
            .unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
        parse_source(&formatted, Some(&path)).unwrap();
        assert_eq!(
            format_source(&formatted, &path).unwrap(),
            formatted,
            "{}",
            path.display()
        );
        checked += 1;
    }
    assert!(checked >= 100, "corpus unexpectedly small: {checked}");
}

#[test]
fn formatted_functions_branches_keys_and_literals_run_in_both_backends() {
    let directory = std::env::temp_dir().join(format!("soplang_formatter_{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("main.sop");
    let original = r#"// Keep this comment and the complete anonymous body.
door f=hawl(n){celi n+1}
door object={"content-type":"text/plain"} /* Keep braces { } in comments. */
haddii(been){qor("wrong")} kale(run){qor(f(2))} ugudambeyn{qor("wrong")}
qor(object["content-type"])
qor(e"line\nnext", r"C:\tmp")
qor(f"value {f(3)}")
hawl identity(teed[abn]? value=maran): teed[abn]? { celi value }
hubi(identity(),maran)
"#;
    let formatted = format_source(original, &path).unwrap();
    assert!(formatted.contains("// Keep this comment"));
    assert!(formatted.contains("/* Keep braces { } in comments. */"));
    assert!(formatted.contains("hawl(n) { celi n + 1 }"));
    assert!(formatted.contains("\"content-type\""));
    let expected = "3\ntext/plain\nline\nnext C:\\tmp\nvalue 4\n";
    for source in [original, formatted.as_str()] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
    let binary = directory.join(format!("formatted{}", std::env::consts::EXE_SUFFIX));
    let built = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("--build")
        .arg(&path)
        .arg("-o")
        .arg(&binary)
        .arg("-q")
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let output = Command::new(binary).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    fs::write(&path, "door f = hawl(\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("fmt")
        .arg(&path)
        .arg("--qor")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&path).unwrap(), "door f = hawl(\n");
    fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[test]
fn formatting_through_a_symlink_preserves_the_link_and_file_permissions() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let directory =
        std::env::temp_dir().join(format!("soplang_formatter_link_{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("source.sop");
    let link = directory.join("link.sop");
    fs::write(&path, "door n=1\nqor(n)\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    symlink(&path, &link).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("fmt")
        .arg(&link)
        .arg("--qor")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(fs::read_to_string(&path).unwrap().contains("door n = 1"));
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("fmt")
        .arg(&link)
        .arg("--qor")
        .output()
        .unwrap();
    assert!(!output.status.success());
    fs::remove_dir_all(directory).unwrap();
}
