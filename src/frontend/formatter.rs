//! Format source whitespace while preserving every token and comment.

use std::path::Path;

use super::trivia::{lossless_tokens, PieceKind};
use super::{parse_source, Lexer, TokenType as K};
use crate::error::{runtime_error, SoplangError};

pub fn format_source(source: &str, path: &Path) -> Result<String, SoplangError> {
    parse_source(source, Some(path))?;
    let pieces = lossless_tokens(source)?;
    let mut output = String::new();
    let mut depth = 0usize;
    let mut breaks = 0usize;
    let mut previous: Option<K> = None;
    let mut previous_prefix = false;
    for (index, piece) in pieces.iter().enumerate() {
        let raw = &source[piece.range.clone()];
        if piece.kind == PieceKind::Whitespace {
            breaks = breaks.max(raw.bytes().filter(|byte| *byte == b'\n').count().min(2));
            continue;
        }
        let kind = match &piece.kind {
            PieceKind::Token(kind) => Some(kind),
            _ => None,
        };
        if kind.is_some_and(closes) {
            depth = depth.saturating_sub(1);
        }
        if !output.is_empty() && breaks > 0 {
            output.extend(std::iter::repeat_n('\n', breaks));
        }
        if output.is_empty() || output.ends_with('\n') {
            output.push_str(&"    ".repeat(depth));
        } else {
            let next_is_colon = pieces[index + 1..]
                .iter()
                .find(|piece| piece.kind != PieceKind::Whitespace)
                .is_some_and(|piece| piece.kind == PieceKind::Token(K::Colon));
            if needs_space(previous.as_ref(), kind, previous_prefix, next_is_colon) {
                output.push(' ');
            }
        }
        // Preserve prefixes, delimiters, escapes, interpolation expressions and
        // multiline literal contents exactly. Comments are protected too.
        output.push_str(raw);
        breaks = 0;
        if let Some(kind) = kind {
            if opens(kind) {
                depth += 1;
            }
            previous_prefix = prefix(kind, previous.as_ref(), previous_prefix);
            previous = Some(kind.clone());
        } else {
            previous = None;
            previous_prefix = false;
        }
    }
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }

    // This also protects future syntax additions: never write output that
    // cannot be parsed or whose token bytes changed under a spacing rule.
    let before = Lexer::new(source).tokenize()?;
    let after = Lexer::new(&output).tokenize()?;
    if before.len() != after.len()
        || before.iter().zip(&after).any(|(left, right)| {
            left.kind != right.kind
                || source[left.start..left.end] != output[right.start..right.end]
        })
    {
        return Err(runtime_error(
            "Qaabayntu waxay beddeli lahayd macnaha qoraalka; faylka lama beddelin.",
            0,
            0,
        ));
    }
    parse_source(&output, Some(path))?;
    Ok(output)
}

fn opens(kind: &K) -> bool {
    matches!(kind, K::LParen | K::LBracket | K::LBrace)
}
fn closes(kind: &K) -> bool {
    matches!(kind, K::RParen | K::RBracket | K::RBrace)
}
fn operator(kind: &K) -> bool {
    matches!(
        kind,
        K::Plus
            | K::Minus
            | K::Star
            | K::Slash
            | K::Modulo
            | K::EqEq
            | K::NotEq
            | K::Greater
            | K::Less
            | K::GreaterEq
            | K::LessEq
            | K::And
            | K::Or
            | K::Assign
            | K::Union
    )
}
fn prefix(kind: &K, previous: Option<&K>, previous_prefix: bool) -> bool {
    kind == &K::Not
        || (matches!(kind, K::Plus | K::Minus | K::Star)
            && (previous.is_none()
                || previous.is_some_and(|kind| {
                    opens(kind) || operator(kind) || matches!(kind, K::Comma | K::Colon | K::Celi)
                })
                || previous_prefix))
}

fn needs_space(
    previous: Option<&K>,
    next: Option<&K>,
    previous_prefix: bool,
    next_is_colon: bool,
) -> bool {
    let (Some(previous), Some(next)) = (previous, next) else {
        return true;
    };
    if matches!(
        next,
        K::RParen | K::RBracket | K::Comma | K::Semicolon | K::Dot | K::Optional
    ) {
        return false;
    }
    if next == &K::Colon {
        return previous != &K::Colon && next_is_colon;
    }
    if matches!(previous, K::LParen | K::LBracket | K::Dot) || previous_prefix {
        return false;
    }
    if previous == &K::LBrace && next == &K::RBrace {
        return false;
    }
    if next == &K::LParen {
        return !matches!(
            previous,
            K::Identifier
                | K::Qor
                | K::Gelin
                | K::Hawl
                | K::Abn
                | K::Jajab
                | K::Qoraal
                | K::Bool
                | K::Teed
                | K::Walax
                | K::RParen
                | K::RBracket
                | K::RBrace
        );
    }
    if next == &K::LBracket
        && !operator(previous)
        && previous != &K::Comma
        && previous != &K::Colon
        && previous != &K::Celi
    {
        return false;
    }
    true
}
