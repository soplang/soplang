//! Character-by-character lexer for Soplang source.

use std::iter::Peekable;
use std::str::Chars;

use crate::error::{codes, lexer_error_ex, ErrorMeta, SoplangError};
use super::token::{Token, TokenType};

pub struct Lexer<'a> {
    #[allow(dead_code)]
    source:  &'a str,
    chars:   Peekable<Chars<'a>>,
    line:    usize,
    col:     usize,
    /// Byte offset of `current` in `source`.
    index:   usize,
    current: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars().peekable();
        let current = chars.next();
        Self { source, chars, line: 1, col: 1, index: 0, current }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, SoplangError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenType::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, SoplangError> {
        while let Some(c) = self.current {
            if c.is_whitespace() {
                self.skip_whitespace();
                continue;
            }
            if c == '/' {
                if self.skip_comment()? {
                    continue;
                }
            }
            let line = self.line;
            let col = self.col;
            let start = self.index;
            if c.is_alphabetic() || c == '_' {
                return Ok(self.read_identifier(line, col, start));
            }
            if c.is_ascii_digit() {
                return Ok(self.read_number(line, col, start));
            }
            if c == '"' || c == '\'' {
                return self.read_string(c, line, col, start);
            }
            if c == '=' {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token::new(TokenType::EqEq, "==", line, col, start));
                }
                return Ok(Token::new(TokenType::Assign, "=", line, col, start));
            }
            if c == '>' {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token::new(TokenType::GreaterEq, ">=", line, col, start));
                }
                return Ok(Token::new(TokenType::Greater, ">", line, col, start));
            }
            if c == '<' {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token::new(TokenType::LessEq, "<=", line, col, start));
                }
                return Ok(Token::new(TokenType::Less, "<", line, col, start));
            }
            if c == '!' {
                self.advance();
                if self.current == Some('=') {
                    self.advance();
                    return Ok(Token::new(TokenType::NotEq, "!=", line, col, start));
                }
                return Ok(Token::new(TokenType::Not, "!", line, col, start));
            }
            if c == '&' {
                self.advance();
                if self.current == Some('&') {
                    self.advance();
                    return Ok(Token::new(TokenType::And, "&&", line, col, start));
                }
                return Err(lexer_error_ex(
                    format!("Xaraf aan la filayn: {}", c),
                    line, col,
                    ErrorMeta::default().with_code(codes::E001_UNEXPECTED_CHAR),
                ));
            }
            if c == '|' {
                self.advance();
                if self.current == Some('|') {
                    self.advance();
                    return Ok(Token::new(TokenType::Or, "||", line, col, start));
                }
                return Err(lexer_error_ex(
                    format!("Xaraf aan la filayn: {}", c),
                    line, col,
                    ErrorMeta::default().with_code(codes::E001_UNEXPECTED_CHAR),
                ));
            }
            let (kind, lexeme) = match c {
                '+' => (TokenType::Plus, "+"),
                '-' => (TokenType::Minus, "-"),
                '*' => (TokenType::Star, "*"),
                '/' => (TokenType::Slash, "/"),
                '%' => (TokenType::Modulo, "%"),
                '(' => (TokenType::LParen, "("),
                ')' => (TokenType::RParen, ")"),
                '{' => (TokenType::LBrace, "{"),
                '}' => (TokenType::RBrace, "}"),
                '[' => (TokenType::LBracket, "["),
                ']' => (TokenType::RBracket, "]"),
                ',' => (TokenType::Comma, ","),
                ':' => (TokenType::Colon, ":"),
                ';' => (TokenType::Semicolon, ";"),
                '.' => (TokenType::Dot, "."),
                _ => {
                    return Err(lexer_error_ex(
                        format!("Xaraf aan la aqoon: {}", c),
                        line, col,
                        ErrorMeta::default().with_code(codes::E001_UNEXPECTED_CHAR),
                    ));
                }
            };
            self.advance();
            return Ok(Token::new(kind, lexeme, line, col, start));
        }
        Ok(Token::new(TokenType::Eof, "", self.line, self.col, self.index))
    }

    fn advance(&mut self) {
        if let Some(c) = self.current {
            self.index += c.len_utf8();
        }
        if self.current == Some('\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        self.current = self.chars.next();
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn skip_whitespace(&mut self) {
        while self.current.map_or(false, |c| c.is_whitespace()) {
            self.advance();
        }
    }

    fn skip_comment(&mut self) -> Result<bool, SoplangError> {
        if self.peek() != Some('/') && self.peek() != Some('*') {
            return Ok(false);
        }
        if self.peek() == Some('/') {
            self.advance();
            self.advance();
            while self.current.is_some() && self.current != Some('\n') {
                self.advance();
            }
            if self.current == Some('\n') {
                self.advance();
            }
            return Ok(true);
        }
        self.advance();
        self.advance();
        while self.current.is_some() {
            if self.current == Some('*') && self.peek() == Some('/') {
                self.advance();
                self.advance();
                return Ok(true);
            }
            self.advance();
        }
        Err(lexer_error_ex(
            "Faallo aan la xidhin",
            self.line, self.col,
            ErrorMeta::default().with_code(codes::E003_UNTERMINATED_COMMENT),
        ))
    }

    fn read_identifier(&mut self, start_line: usize, start_col: usize, start: usize) -> Token {
        let mut s = String::new();
        while self.current.map_or(false, |c| c.is_alphanumeric() || c == '_') {
            s.push(self.current.unwrap());
            self.advance();
        }
        let kind = keyword(&s).unwrap_or(TokenType::Identifier);
        Token::new(kind, s, start_line, start_col, start)
    }

    fn read_number(&mut self, start_line: usize, start_col: usize, start: usize) -> Token {
        let mut s = String::new();
        while self.current.map_or(false, |c| c.is_ascii_digit() || c == '.') {
            s.push(self.current.unwrap());
            self.advance();
        }
        Token::new(TokenType::Number, s, start_line, start_col, start)
    }

    fn read_string(&mut self, quote: char, start_line: usize, start_col: usize, start: usize) -> Result<Token, SoplangError> {
        self.advance();
        let mut s = String::new();
        while self.current.is_some() && self.current != Some(quote) {
            if let Some(c) = self.current {
                s.push(c);
                self.advance();
            }
        }
        if self.current == Some(quote) {
            self.advance();
            Ok(Token::new(TokenType::String, s, start_line, start_col, start))
        } else {
            Err(lexer_error_ex(
                "Qoraal aan la xidhin",
                self.line, self.col,
                ErrorMeta::default().with_code(codes::E002_UNTERMINATED_STRING),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_hello() {
        let source = r#"qor("Salaan, Adduunka!")"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].kind, TokenType::Qor);
        assert_eq!(tokens[2].lexeme, "Salaan, Adduunka!");
    }

    #[test]
    fn unterminated_string_error() {
        let source = r#"qor("unclosed"#;
        let mut lexer = Lexer::new(source);
        assert!(lexer.tokenize().is_err());
    }
}

fn keyword(s: &str) -> Option<TokenType> {
    match s {
        "door" => Some(TokenType::Door),
        "madoor" => Some(TokenType::Madoor),
        "hawl" => Some(TokenType::Hawl),
        "celi" => Some(TokenType::Celi),
        "qor" => Some(TokenType::Qor),
        "gelin" => Some(TokenType::Gelin),
        "haddii" => Some(TokenType::Haddii),
        "haddii_kale" => Some(TokenType::HaddiiKale),
        "ugudambeyn" => Some(TokenType::Ugudambeyn),
        "dooro" => Some(TokenType::Dooro),
        "xaalad" => Some(TokenType::Xaalad),
        "kuceli" => Some(TokenType::Kuceli),
        "intay" => Some(TokenType::Intay),
        "jooji" => Some(TokenType::Jooji),
        "soco" => Some(TokenType::Soco),
        "fasax" => Some(TokenType::Fasax),
        "qabo" => Some(TokenType::Qabo),
        "keen" => Some(TokenType::Keen),
        "qaab" => Some(TokenType::Qaab),
        "dhaxal" => Some(TokenType::Dhaxal),
        "cusub" => Some(TokenType::Cusub),
        "nafta" => Some(TokenType::Nafta),
        "waalid" => Some(TokenType::Waalid),
        "abn" => Some(TokenType::Abn),
        "jajab" => Some(TokenType::Jajab),
        "qoraal" => Some(TokenType::Qoraal),
        "bool" => Some(TokenType::Bool),
        "teed" => Some(TokenType::Teed),
        "walax" => Some(TokenType::Walax),
        "run" => Some(TokenType::True),
        "been" => Some(TokenType::False),
        "maran" | "null" => Some(TokenType::Null),
        _ => None,
    }
}
