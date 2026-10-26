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
        let prefixed = matches!(bytes[i], b'e' | b'r' | b'f')
            && matches!(bytes.get(i + 1), Some(b'\'' | b'"'))
            && !text[..i]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_alphanumeric() || ch == '_');
        let quote_at = if prefixed { i + 1 } else { i };
        if prefixed && bytes[i] == b'f' {
            // Reuse interpolation's nested-expression scanner, including strings
            // with the same quote character. Incomplete editor input stays protected.
            i = Lexer::new(&text[start..])
                .next_token()
                .map(|token| start + token.end)
                .unwrap_or(bytes.len());
            ranges.push(start..i);
        } else if prefixed || matches!(bytes[i], b'\'' | b'"') {
            let escaped = prefixed && bytes[i] == b'e';
            let quote = bytes[quote_at];
            let triple = bytes.get(quote_at..quote_at + 3) == Some(&[quote, quote, quote]);
            let width = if triple { 3 } else { 1 };
            i = quote_at + width;
            while i < bytes.len() {
                if escaped && bytes[i] == b'\\' {
                    i = (i + 2).min(bytes.len());
                } else if bytes[i] == quote
                    && (!triple || bytes.get(i..i + 3) == Some(&[quote, quote, quote]))
                {
                    i += width;
                    break;
                } else {
                    i += 1;
                }
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
        let end = token.end;
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
