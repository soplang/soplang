//! Render public APIs from original source signatures and documentation comments.
use crate::error::SoplangError;
use crate::frontend::{ast::Stmt, Lexer, Token, TokenType as K};
use std::collections::HashSet;
use std::path::Path;

pub(super) fn render(
    source: &str,
    path: &Path,
    stmts: &[Stmt],
    private: bool,
) -> Result<String, SoplangError> {
    let tokens = Lexer::new(source).tokenize()?;
    let public: HashSet<_> = crate::frontend::imports::export_names(stmts)?
        .into_iter()
        .collect();
    let mut out = format!(
        "# {}\n\n",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    if !public.is_empty() {
        let mut names: Vec<_> = public.iter().cloned().collect();
        names.sort();
        out.push_str(&format!("Exports: `{}`\n\n", names.join("`, `")));
    }
    for stmt in stmts {
        declaration(stmt, source, &tokens, &public, private, false, 2, &mut out);
    }
    Ok(out)
}

fn signature<'a>(stmt: &Stmt, source: &'a str, tokens: &[Token]) -> Option<(&'a str, usize)> {
    let Stmt::Located { span, .. } = stmt else {
        return None;
    };
    let start = tokens
        .iter()
        .position(|token| token.line == span.line && token.col == span.col)?;
    let mut parens = 0usize;
    let mut brackets = 0usize;
    for token in &tokens[start..] {
        match token.kind {
            K::LParen => parens += 1,
            K::RParen => parens = parens.saturating_sub(1),
            K::LBracket => brackets += 1,
            K::RBracket => brackets = brackets.saturating_sub(1),
            K::LBrace if parens == 0 && brackets == 0 => {
                return Some((
                    source[tokens[start].start..token.start].trim_end(),
                    span.line,
                ))
            }
            _ => {}
        }
    }
    None
}
fn comment(source: &str, line: usize) -> String {
    let lines: Vec<_> = source.lines().take(line.saturating_sub(1)).collect();
    let mut comments = Vec::new();
    for line in lines.iter().rev() {
        let Some(text) = line.trim_start().strip_prefix("///") else {
            break;
        };
        comments.push(text.strip_prefix(' ').unwrap_or(text));
    }
    comments.reverse();
    comments.join("\n")
}
fn code(out: &mut String, signature: &str) {
    // A default string can contain Markdown fences. Choose a longer delimiter.
    let mut fence = String::from("```");
    while signature.contains(&fence) {
        fence.push('`');
    }
    out.push_str(&format!("{fence}soplang\n{signature}\n{fence}\n\n"));
}
fn declaration(
    stmt: &Stmt,
    source: &str,
    tokens: &[Token],
    public: &HashSet<String>,
    private: bool,
    member: bool,
    level: usize,
    out: &mut String,
) {
    let name = match stmt.unlocated() {
        Stmt::FuncDef { name, .. } | Stmt::ClassDef { name, .. } | Stmt::VarDecl { name, .. } => {
            name
        }
        Stmt::Test {
            name,
            expect_error,
            expected_code,
            ..
        } => {
            out.push_str(&format!("{} tijaabo\n\n", "#".repeat(level)));
            code(
                out,
                &format!(
                    "tijaabo {}{}{}",
                    crate::frontend::ast::Literal::Str(name.clone()),
                    if *expect_error { " qalad" } else { "" },
                    expected_code
                        .as_ref()
                        .map(|value| format!(
                            "({})",
                            crate::frontend::ast::Literal::Str(value.clone())
                        ))
                        .unwrap_or_default()
                ),
            );
            return;
        }
        _ => return,
    };
    if !private && (member && name.starts_with('_') || !member && !public.contains(name)) {
        return;
    }
    out.push_str(&format!(
        "{} {} {}\n\n",
        "#".repeat(level),
        match stmt.unlocated() {
            Stmt::FuncDef { .. } => "hawl",
            Stmt::ClassDef { .. } => "qaab",
            _ => "doorsoome",
        },
        name
    ));
    if let Stmt::VarDecl {
        type_ann, is_const, ..
    } = stmt.unlocated()
    {
        code(
            out,
            &format!(
                "{}{} {name}",
                if *is_const { "madoor " } else { "" },
                type_ann
            ),
        );
        if let Stmt::Located { span, .. } = stmt {
            let doc = comment(source, span.line);
            if !doc.is_empty() {
                out.push_str(&doc);
                out.push_str("\n\n");
            }
        }
    } else if let Some((signature, line)) = signature(stmt, source, tokens) {
        code(out, signature);
        let doc = comment(source, line);
        if !doc.is_empty() {
            out.push_str(&doc);
            out.push_str("\n\n");
        }
    }
    if let Stmt::ClassDef { body, .. } = stmt.unlocated() {
        for method in body {
            declaration(
                method,
                source,
                tokens,
                public,
                private,
                true,
                level + 1,
                out,
            );
        }
    }
}
