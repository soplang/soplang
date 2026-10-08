//! LLVM IR linked to the runtime. The binary does not call `run_source`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use soplang::backend::llvm_ir::{emit_llvm_ir, link_llvm_binary};
use soplang::{analyze, HirLowering, Lexer, Parser};

fn temp_bin(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let mut p = std::env::temp_dir();
    p.push(format!("soplang_{name}_{}_{stamp}", std::process::id()));
    p
}

#[test]
fn test_llvm_ir_prints_without_run_source() {
    let source = r#"
        hawl add(a, b) {
            celi a + b
        }
        abn n = 0
        intay (n < 3) {
            haddii (n == 1) {
                qor(add(n, 10))
            } ugudambeyn {
                qor(n)
            }
            n = n + 1
        }
    "#;
    let tokens = Lexer::new(source).tokenize().expect("lex");
    let stmts = Parser::new(tokens).parse().expect("parse");
    let sym = analyze(&stmts).expect("analyze");
    let hir = HirLowering::lower(&sym, &stmts);
    let ir = emit_llvm_ir(&hir).unwrap_or_else(|e| panic!("{e}"));
    assert!(ir.contains("define i32 @main()"), "IR has no LLVM main");
    assert!(
        ir.contains("@soplang_add"),
        "arithmetic does not call the runtime"
    );
    assert!(
        !ir.contains("run_source"),
        "LLVM binary must not go through run_source"
    );

    let bin = temp_bin("llvm_loop");
    link_llvm_binary(&ir, &bin).unwrap_or_else(|e| panic!("{e}"));
    let out = Command::new(&bin).output().expect("run llvm binary");
    let stdout = String::from_utf8(out.stdout).expect("utf8");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = fs::remove_file(&bin);
    assert!(
        out.status.success(),
        "binary failed: {stderr}\nstdout: {stdout}"
    );
    assert_eq!(stdout, "0\n11\n2\n");
}
