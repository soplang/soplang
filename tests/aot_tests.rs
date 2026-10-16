//! Integration tests for AOT build flow (`--build`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn soplang_exe() -> &'static str {
    env!("CARGO_BIN_EXE_soplang")
}

fn temp_bin_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let suffix = format!("{}_{}_{}", name, std::process::id(), chrono_like_now());
    #[cfg(windows)]
    {
        p.push(format!("{}.exe", suffix));
    }
    #[cfg(not(windows))]
    {
        p.push(suffix);
    }
    p
}

fn chrono_like_now() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn run_and_capture(path: &Path) -> String {
    let out = Command::new(path).output().expect("run generated binary");
    String::from_utf8(out.stdout).expect("utf8 stdout")
}

#[test]
fn test_aot_build_hello_matches_expected() {
    let out_bin = temp_bin_path("hello_aot");
    let status = Command::new(soplang_exe())
        .arg("--build")
        .arg("examples/01_hello.sop")
        .arg("-o")
        .arg(&out_bin)
        .status()
        .expect("run soplang --build");
    assert!(status.success(), "AOT build failed");

    let actual = run_and_capture(&out_bin);
    assert_eq!(actual, "Salaan, Adduunka!\n");

    let _ = fs::remove_file(out_bin);
}

#[test]
fn test_aot_build_import_inlines_keen() {
    let out_bin = temp_bin_path("import_aot");
    let status = Command::new(soplang_exe())
        .arg("--build")
        .arg("examples/16_import.sop")
        .arg("-o")
        .arg(&out_bin)
        .status()
        .expect("run soplang --build");
    assert!(status.success(), "AOT build of import example failed");

    let actual = run_and_capture(&out_bin);
    assert_eq!(actual, "6\n");

    let _ = fs::remove_file(out_bin);
}

#[test]
fn test_aot_build_opt_level_0_runs() {
    let out_bin = temp_bin_path("hello_aot_o0");
    let status = Command::new(soplang_exe())
        .arg("--build")
        .arg("examples/01_hello.sop")
        .arg("--opt-level")
        .arg("0")
        .arg("-o")
        .arg(&out_bin)
        .status()
        .expect("run soplang --build --opt-level 0");
    assert!(status.success(), "AOT build (opt-level=0) failed");

    let actual = run_and_capture(&out_bin);
    assert_eq!(actual, "Salaan, Adduunka!\n");

    let _ = fs::remove_file(out_bin);
}

#[test]
fn test_aot_build_compiles_source_instead_of_embedding_it() {
    const MARKER: &str = "SOPLANG_SOURCE_ONLY_MARKER_6e47f2b1";
    let source_path = temp_bin_path("llvm_source").with_extension("sop");
    let out_bin = temp_bin_path("llvm_compiled");
    fs::write(
        &source_path,
        format!("// {MARKER}\nabn n = 6\nqor(n * 7)\n"),
    )
    .expect("write source");
    let output = Command::new(soplang_exe())
        .arg("--build")
        .arg(&source_path)
        .arg("--quiet")
        .arg("-o")
        .arg(&out_bin)
        .output()
        .expect("run soplang --build");
    assert!(
        output.status.success(),
        "AOT build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(run_and_capture(&out_bin), "42\n");
    let binary = fs::read(&out_bin).expect("read generated binary");
    assert!(
        !binary.windows(MARKER.len()).any(|w| w == MARKER.as_bytes()),
        "source comment was embedded in the binary"
    );
    let _ = fs::remove_file(source_path);
    let _ = fs::remove_file(out_bin);
}
