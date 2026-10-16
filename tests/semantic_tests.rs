//! Phase 1 (COMPILER_PLAN): semantic analysis passes on all example files.

use std::fs;
use std::path::Path;

use soplang::{semantic, Lexer, Parser};

fn run_semantic(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let tokens = Lexer::new(&source).tokenize().map_err(|e| e.to_string())?;
    let stmts = Parser::new(tokens).parse().map_err(|e| e.to_string())?;
    semantic::analyze(&stmts).map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn test_semantic_all_examples() {
    let examples_dir = Path::new("examples");
    if !examples_dir.is_dir() {
        return;
    }
    let mut sop_files: Vec<_> = fs::read_dir(examples_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sop"))
        .map(|e| e.path())
        .collect();
    sop_files.sort();

    for path in sop_files {
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        // Skip examples that are intentionally type-error tests.
        if name.contains("type_error") || name.contains("reassignment") {
            continue;
        }
        let result = run_semantic(&path);
        assert!(
            result.is_ok(),
            "semantic analysis should pass for {}: {:?}",
            path.display(),
            result.err()
        );
    }
}

#[test]
fn test_semantic_mangles_class_methods() {
    let source = r#"
        qaab A {
            hawl foo() { celi 1 }
        }
        qaab B dhaxal A {
            hawl bar() { celi 2 }
        }
    "#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    let sym = semantic::analyze(&stmts).unwrap();
    assert!(sym.functions.iter().any(|f| f.name == "A::foo" && f.is_method));
    assert!(sym.functions.iter().any(|f| f.name == "B::bar" && f.is_method));
    assert_eq!(sym.classes.get("B").unwrap().parent.as_deref(), Some("A"));
}

#[test]
fn test_semantic_rejects_cyclic_parent() {
    let source = r#"
        qaab A dhaxal B { hawl foo() { celi 1 } }
        qaab B dhaxal A { hawl bar() { celi 2 } }
    "#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    let err = semantic::analyze(&stmts).unwrap_err().to_string();
    assert!(err.contains("meel-wareeg"), "{err}");
}

#[test]
fn test_semantic_rejects_self_parent() {
    let source = r#"
        qaab A dhaxal A { hawl foo() { celi 1 } }
    "#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    let err = semantic::analyze(&stmts).unwrap_err().to_string();
    assert!(err.contains("meel-wareeg"), "{err}");
}

#[test]
fn test_semantic_rejects_waalid_without_parent() {
    let source = r#"
        qaab A {
            hawl cod() { celi waalid.cod() }
        }
    "#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    assert!(semantic::analyze(&stmts).is_err());
}

#[test]
fn test_semantic_class_typed_var() {
    let source = r#"
        qaab Xayawaan { hawl dhaw(magac) { nafta.magac = magac } }
        qaab Bisad dhaxal Xayawaan { hawl dhaw(magac) { nafta.magac = magac } }
        qaab Kale { hawl dhaw() {} }
        Bisad b = cusub Bisad("Mia")
        Xayawaan x = cusub Bisad("Luna")
    "#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    assert!(semantic::analyze(&stmts).is_ok());

    let bad = r#"
        qaab Bisad { hawl dhaw(magac) { nafta.magac = magac } }
        qaab Kale { hawl dhaw() {} }
        Bisad b = cusub Kale()
    "#;
    let tokens = Lexer::new(bad).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    assert!(semantic::analyze(&stmts).is_err());

    let missing = r#"
        Missing m = 1
    "#;
    let tokens = Lexer::new(missing).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    let err = semantic::analyze(&stmts).unwrap_err().to_string();
    assert!(err.contains("Missing"), "{err}");
}

#[test]
fn test_semantic_rejects_unknown_parent() {
    let source = r#"
        qaab Child dhaxal Missing {
            hawl foo() { celi 1 }
        }
    "#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    assert!(semantic::analyze(&stmts).is_err());
}

#[test]
fn test_import_resolves_and_merges() {
    use std::path::Path;
    use soplang::frontend::imports;
    use soplang::Stmt;

    let main = Path::new("examples/16_import.sop");
    let source = std::fs::read_to_string(main).unwrap();
    let tokens = Lexer::new(&source).tokenize().unwrap();
    let stmts = imports::resolve_imports(Parser::new(tokens).parse().unwrap(), Some(main)).unwrap();
    assert!(stmts.iter().any(|s| matches!(s, Stmt::FuncDef { name, .. } if name == "laba")));
    let sym = semantic::analyze(&stmts).unwrap();
    assert!(sym.functions.iter().any(|f| f.name == "laba"));
}
