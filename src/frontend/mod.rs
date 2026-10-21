//! Frontend: source text → tokens → AST.

pub mod ast;
pub mod formatter;
pub mod imports;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod trivia;

pub use ast::{Expr, Literal, Param, Stmt, TypeAnnotation};
pub use lexer::Lexer;
pub use parser::Parser;
pub use token::{Token, TokenType};

/// Attach the original file and line without changing a lexer's/parser's column.
pub fn source_error(
    error: crate::error::SoplangError,
    source: &str,
    path: Option<&std::path::Path>,
) -> crate::error::SoplangError {
    let (line, col) = error.location();
    crate::error::SourceSpan {
        file: Some(
            path.unwrap_or_else(|| std::path::Path::new("<command>"))
                .to_path_buf(),
        ),
        line,
        col,
        end_col: col,
        source_line: source
            .split('\n')
            .nth(line.saturating_sub(1))
            .unwrap_or("")
            .to_string(),
    }
    .attach(error)
}

pub fn parse_source(
    source: &str,
    path: Option<&std::path::Path>,
) -> Result<Vec<Stmt>, crate::error::SoplangError> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|e| source_error(e, source, path))?;
    Parser::new(tokens)
        .with_source(source, path)
        .parse()
        .map_err(|e| source_error(e, source, path))
}

/// Wrap the AST, rather than source text, so expression echo keeps its columns.
pub fn parse_snippet(source: &str) -> Result<Vec<Stmt>, crate::error::SoplangError> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|e| source_error(e, source, None))?;
    let mut parser = Parser::new(tokens.clone()).with_source(source, None);
    let expression = parser.parse_single_expression();
    if let Ok(expr) = expression {
        if matches!(expr.unlocated(), Expr::Call { name, .. } if name == "qor") {
            return Ok(vec![Stmt::Expr(expr)]);
        }
        return Ok(vec![Stmt::Expr(Expr::Call {
            name: "qor".into(),
            args: vec![expr],
        })]);
    }
    let expression_only = tokens.first().is_some_and(|token| {
        matches!(
            token.kind,
            TokenType::Number
                | TokenType::String
                | TokenType::FormattedString
                | TokenType::True
                | TokenType::False
                | TokenType::Null
                | TokenType::Minus
                | TokenType::Plus
                | TokenType::Not
                | TokenType::LParen
                | TokenType::LBracket
                | TokenType::Cusub
        )
    });
    if expression_only {
        return Err(source_error(expression.unwrap_err(), source, None));
    }
    Parser::new(tokens)
        .with_source(source, None)
        .parse()
        .map_err(|e| source_error(e, source, None))
}

/// Best-effort editor tree. Repair incomplete delimiters only for the editor tree;
/// diagnostics and normal compilation always parse the original source.
pub fn parse_partial(source: &str, path: Option<&std::path::Path>) -> Vec<Stmt> {
    let Ok(tokens) = Lexer::new(source).tokenize() else { return Vec::new(); };
    let mut closers = Vec::new();
    for token in &tokens { match token.kind {
        TokenType::LParen => closers.push(')'), TokenType::LBrace => closers.push('}'),
        TokenType::LBracket => closers.push(']'), TokenType::RParen|TokenType::RBrace|TokenType::RBracket => {closers.pop();}, _=>()
    } }
    let tail: String = closers.iter().rev().collect();
    let base = source.trim_end().trim_end_matches('.');
    for middle in ["", " maran"] {
        let repaired = format!("{base}{middle}{tail}");
        if let Ok(tree) = parse_source(&repaired, path) { return tree; }
    }
    Parser::new(tokens).with_source(source, path).parse_partial()
}
