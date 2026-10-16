//! Integration tests: parse snippets, assert AST structure.

use soplang::{Expr, Lexer, Parser, Stmt};

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
