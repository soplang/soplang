//! Lossless source pieces for editor tooling. Token bytes are never reconstructed.
use super::{Lexer, TokenType};
use crate::error::SoplangError;
use std::ops::Range;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Token(TokenType),
    Whitespace,
    LineComment,
    BlockComment,
}
#[derive(Clone, Debug)]
pub struct SourcePiece {
    pub kind: PieceKind,
    pub range: Range<usize>,
}
/// Best-effort protected spans, including unfinished strings and comments.
pub fn protected_ranges(text: &str) -> Vec<Range<usize>> {
    let mut ranges = vec![];
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let quote = bytes[i];
            i += 1;
            while i < bytes.len() && bytes[i] != quote {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            ranges.push(start..i);
        } else if bytes.get(i..i + 2) == Some(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            ranges.push(start..i);
        } else if bytes.get(i..i + 2) == Some(b"/*") {
            i += 2;
            while i + 1 < bytes.len() && &bytes[i..i + 2] != b"*/" {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            ranges.push(start..i);
        } else {
            i += 1;
        }
    }
    ranges
}
pub fn lossless_tokens(text: &str) -> Result<Vec<SourcePiece>, SoplangError> {
    let tokens = Lexer::new(text).tokenize()?;
    let mut pieces = vec![];
    let mut cursor = 0;
    for token in tokens {
        if token.start > cursor {
            let gap = &text[cursor..token.start];
            let mut local = 0;
            for protected in protected_ranges(gap) {
                if protected.start > local {
                    pieces.push(SourcePiece {
                        kind: PieceKind::Whitespace,
                        range: cursor + local..cursor + protected.start,
                    });
                }
                let kind = if gap[protected.clone()].starts_with("//") {
                    PieceKind::LineComment
                } else {
                    PieceKind::BlockComment
                };
                pieces.push(SourcePiece {
                    kind,
                    range: cursor + protected.start..cursor + protected.end,
                });
                local = protected.end;
            }
            if local < gap.len() {
                pieces.push(SourcePiece {
                    kind: PieceKind::Whitespace,
                    range: cursor + local..token.start,
                });
            }
        }
        let end =
            token.start + token.lexeme.len() + usize::from(token.kind == TokenType::String) * 2;
        if token.kind != TokenType::Eof {
            pieces.push(SourcePiece {
                kind: PieceKind::Token(token.kind),
                range: token.start..end,
            });
        }
        cursor = end;
    }
    Ok(pieces)
}
