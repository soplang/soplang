//! Integration tests: tokenise known inputs, assert token types and lexemes.

use soplang::{Lexer, TokenType};

#[test]
fn test_hello_tokens() {
    let source = r#"qor("Salaan, Adduunka!")"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].kind, TokenType::Qor);
    assert_eq!(tokens[1].kind, TokenType::LParen);
    assert_eq!(tokens[2].kind, TokenType::String);
    assert_eq!(tokens[2].lexeme, "Salaan, Adduunka!");
    assert_eq!(tokens[3].kind, TokenType::RParen);
    assert_eq!(tokens[4].kind, TokenType::Eof);
}

#[test]
fn test_door_assign_number() {
    let source = "door x = 1 + 2";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenType::Door);
    assert_eq!(tokens[1].kind, TokenType::Identifier);
    assert_eq!(tokens[1].lexeme, "x");
    assert_eq!(tokens[2].kind, TokenType::Assign);
    assert_eq!(tokens[3].kind, TokenType::Number);
    assert_eq!(tokens[3].lexeme, "1");
    assert_eq!(tokens[4].kind, TokenType::Plus);
    assert_eq!(tokens[5].kind, TokenType::Number);
    assert_eq!(tokens[5].lexeme, "2");
}

#[test]
fn test_keywords_run_been_maran() {
    let source = "run been maran null";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenType::True);
    assert_eq!(tokens[1].kind, TokenType::False);
    assert_eq!(tokens[2].kind, TokenType::Null);
    assert_eq!(tokens[3].kind, TokenType::Null);
}

#[test]
fn test_final_keyword_vocabulary() {
    let source = concat!(
        "hawl celi qor gelin kuceli intay soco dooro xaalad ugudambeyn ",
        "abn jajab bool teed walax maran"
    );
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenType::Hawl,
            TokenType::Celi,
            TokenType::Qor,
            TokenType::Gelin,
            TokenType::Kuceli,
            TokenType::Intay,
            TokenType::Soco,
            TokenType::Dooro,
            TokenType::Xaalad,
            TokenType::Ugudambeyn,
            TokenType::Abn,
            TokenType::Jajab,
            TokenType::Bool,
            TokenType::Teed,
            TokenType::Walax,
            TokenType::Null,
            TokenType::Eof,
        ]
    );
}

#[test]
fn test_replaced_keywords_are_identifiers() {
    let source = concat!(
        "howl soo_celi bandhig akhri ku_celi inta_ay sii_wad tiro liis shey ",
        "labadaran waxba kudhow raadso"
    );
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens[..tokens.len() - 1]
        .iter()
        .all(|token| token.kind == TokenType::Identifier));
}

#[test]
fn test_unterminated_string_errors() {
    let source = r#"qor("unclosed"#;
    let mut lexer = Lexer::new(source);
    assert!(lexer.tokenize().is_err());
}
