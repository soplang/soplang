//! Recover editor scopes at EOF without accepting incomplete compiler input.
use crate::frontend;
use crate::frontend::{ast::Stmt, Lexer, Parser, Token, TokenType as K};
use std::collections::HashSet;
use std::path::Path;

pub(super) fn editor_syntax(source: &str, path: &Path) -> (Vec<Stmt>, Vec<Token>, HashSet<usize>) {
    let tokens = Lexer::new(source).tokenize_prefix();
    if !bounded(&tokens, &mut 0) {
        return (Vec::new(), tokens, HashSet::new());
    }
    let mut parser = Parser::new(tokens.clone()).with_source(source, Some(path));
    if let Ok(statements) = parser.parse() {
        return (
            statements,
            tokens,
            parser.contextual_keyword_offsets().clone(),
        );
    }
    let mut repaired = tokens.clone();
    let eof = repaired.pop().unwrap();
    let mut closers = Vec::new();
    let mut valid = true;
    for token in &repaired {
        match token.kind {
            K::LBrace => closers.push(K::RBrace),
            K::LParen => closers.push(K::RParen),
            K::LBracket => closers.push(K::RBracket),
            K::RBrace | K::RParen | K::RBracket => {
                if closers.pop() != Some(token.kind.clone()) {
                    valid = false;
                    break;
                }
            }
            _ => {}
        }
        if closers.len() > 128 {
            valid = false;
            break;
        }
    }
    let mut append = |kind, lexeme: &str| {
        let mut token = Token::new(kind, lexeme, eof.line, eof.col, eof.start);
        token.end = token.start;
        repaired.push(token);
    };
    if valid {
        match tokens.iter().rev().nth(1).map(|token| &token.kind) {
            Some(K::Dot) => append(K::Identifier, "__soplang_incomplete_member"),
            Some(
                K::Assign
                | K::Comma
                | K::LParen
                | K::Plus
                | K::Minus
                | K::Star
                | K::Slash
                | K::And
                | K::Or
                | K::Greater
                | K::Less
                | K::EqEq
                | K::NotEq,
            ) => append(K::Null, "maran"),
            _ => {}
        }
        for closer in closers.into_iter().rev() {
            let text = match closer {
                K::RBrace => "}",
                K::RParen => ")",
                _ => "]",
            };
            append(closer, text);
        }
        repaired.push(eof);
        let mut parser = Parser::new(repaired).with_source(source, Some(path));
        if let Ok(statements) = parser.parse() {
            return (
                statements,
                tokens,
                parser.contextual_keyword_offsets().clone(),
            );
        }
    }
    let mut parser = Parser::new(tokens.clone()).with_source(source, Some(path));
    let statements = parser.parse_partial();
    (
        statements,
        tokens,
        parser.contextual_keyword_offsets().clone(),
    )
}

pub(super) fn bounded(tokens: &[frontend::Token], total: &mut usize) -> bool {
    use frontend::TokenType as K;
    let (mut nesting, mut operators) = (0usize, 0usize);
    for token in tokens {
        *total += 1;
        match token.kind {
            K::LBrace | K::LBracket | K::LParen => nesting += 1,
            K::RBrace | K::RBracket | K::RParen => {
                nesting = nesting.saturating_sub(1);
                operators = 0;
            }
            K::Plus | K::Minus | K::Not | K::Star | K::Slash | K::And | K::Or => operators += 1,
            K::Assign | K::Comma | K::Semicolon => operators = 0,
            _ => {}
        }
        if nesting > 128 || operators > 512 || *total > 100_000 {
            return false;
        }
        for part in &token.format_parts {
            if let frontend::token::FormatPart::Field { tokens, .. } = part {
                if !bounded(tokens, total) {
                    return false;
                }
            }
        }
    }
    true
}
