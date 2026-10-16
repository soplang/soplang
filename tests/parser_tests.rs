//! Integration tests: parse snippets, assert AST structure.

use soplang::{Expr, Lexer, Parser, Stmt};

#[test]
fn test_contextual_kale_branches_and_calls() {
    let source = r#"
        door kale = 1
        hawl f(kale) { celi kale }
        haddii (run) {} kale((1 + 1) == 2) {} kale {}
        haddii (run) {} kale(1)
        dooro (1) { xaalad 1 {} kale {} }
    "#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    assert!(matches!(&stmts[0], Stmt::VarDecl { name, .. } if name == "kale"));
    assert!(
        matches!(&stmts[2], Stmt::If { elseifs, else_body: Some(_), .. } if elseifs.len() == 1)
    );
    assert!(matches!(&stmts[3], Stmt::If { elseifs, else_body: None, .. } if elseifs.is_empty()));
    assert!(matches!(&stmts[4], Stmt::Expr(Expr::Call { name, .. }) if name == "kale"));
    assert!(matches!(
        &stmts[5],
        Stmt::Switch {
            default: Some(_),
            ..
        }
    ));

    for invalid in [
        "haddii (run) {} kale (run) {",
        "haddii (run) {} kale ((run) {}",
        "dooro (1) { kale (run) {} }",
    ] {
        let tokens = Lexer::new(invalid).tokenize().unwrap();
        assert!(Parser::new(tokens).parse().is_err(), "accepted: {invalid}");
    }
}

#[test]
fn test_contextual_tijaabi_preserves_identifiers() {
    let source = r#"
        door tijaabi = 1
        tijaabi = tijaabi + 1
        hawl f(tijaabi) { celi tijaabi }
        hawl tijaabi(x) { celi x }
        tijaabi(2)
        tijaabi { qor(1 / 0) } qabo (e) { qor(e) }
        fasax { qor(1 / 0) } qabo (e) { qor(e) }
    "#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    assert!(matches!(&stmts[0], Stmt::VarDecl { name, .. } if name == "tijaabi"));
    assert!(matches!(&stmts[4], Stmt::Expr(Expr::Call { name, .. }) if name == "tijaabi"));
    assert!(matches!(&stmts[5], Stmt::TryCatch { .. }));
    assert!(matches!(&stmts[6], Stmt::TryCatch { .. }));
}

#[test]
fn test_parse_hello() {
    let source = r#"qor("Salaan, Adduunka!")"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    assert_eq!(stmts.len(), 1);
    if let Stmt::Expr(Expr::Call { name, args }) = &stmts[0] {
        assert_eq!(name, "qor");
        assert_eq!(args.len(), 1);
    } else {
        panic!("expected qor(...) statement");
    }
}

#[test]
fn test_parse_var_decl() {
    let source = "door x = 42";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    assert_eq!(stmts.len(), 1);
    if let Stmt::VarDecl { name, value, .. } = &stmts[0] {
        assert_eq!(name, "x");
        assert!(matches!(value, Expr::Literal(_)));
    } else {
        panic!("expected var decl");
    }
}

#[test]
fn test_optional_semicolons_between_statements_and_blocks() {
    let source = r#"
        ; door x = 1;;
        hawl value() { celi; };
        haddii (run) { qor(x); };
    "#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    assert_eq!(stmts.len(), 3);
}

#[test]
fn test_parse_if_then() {
    let source = r#"
        haddii (run) {
            qor(1)
        }
    "#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::If { .. }));
}

#[test]
fn test_parse_class_and_new() {
    let source = r#"
        qaab Bisad {
            hawl dhaw(magac) {
                nafta.magac = magac
            }
        }
        door x = cusub Bisad("Mia")
    "#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    assert_eq!(stmts.len(), 2);
    assert!(matches!(&stmts[0], Stmt::ClassDef { .. }));
    if let Stmt::VarDecl { value, .. } = &stmts[1] {
        assert!(matches!(value, Expr::New { .. }));
    } else {
        panic!("expected var decl with cusub");
    }
}

#[test]
fn test_parse_import_and_try_catch() {
    let source = r#"
        keen "lib.sop"
        fasax {
            qor(1)
        } qabo (e) {
            qor(e)
        }
    "#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let stmts = Parser::new(tokens).parse().unwrap();
    assert!(matches!(&stmts[0], Stmt::Import(_)));
    if let Stmt::TryCatch { err_var, .. } = &stmts[1] {
        assert_eq!(err_var, "e");
    } else {
        panic!("expected try/catch");
    }
}
