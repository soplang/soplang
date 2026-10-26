//! Character-by-character lexer for Soplang source.

use std::iter::Peekable;
use std::str::Chars;

use super::token::{Token, TokenType};
use crate::error::{codes, lexer_error_ex, ErrorMeta, SoplangError};

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
    /// Byte offset of `current` in `source`.
    index: usize,
    current: Option<char>,
    format_depth: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars().peekable();
        let current = chars.next();
        Self {
            source,
            chars,
            line: 1,
            col: 1,
            index: 0,
            current,
            format_depth: 0,
        }
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

    /// Editor-only prefix tokens. Keep valid earlier tokens when the current
    /// string/comment is unfinished, and stop before the editor token budget.
    /// Compiler tokenization continues to return the original lexical error.
    pub(crate) fn tokenize_prefix(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            match self.next_token() {
                Ok(token) => {
                    let eof = token.kind == TokenType::Eof;
                    tokens.push(token);
                    if eof { return tokens; }
                }
                Err(_) => break,
            }
            if tokens.len() > 100_000 { break; }
        }
        let mut eof = Token::new(TokenType::Eof, "", self.line, self.col, self.index);
        eof.end = eof.start;
        tokens.push(eof);
        tokens
    }

    pub(super) fn next_token(&mut self) -> Result<Token, SoplangError> {
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
            if matches!(c, 'e' | 'r' | 'f') && matches!(self.peek(), Some('"' | '\'')) {
                let escaped = c == 'e';
                self.advance();
                if c == 'f' {
                    return self.read_formatted_string(self.current.unwrap(), line, col, start);
                }
                return self.read_string(self.current.unwrap(), escaped, line, col, start);
            }
            if c.is_alphabetic() || c == '_' {
                return Ok(self.read_identifier(line, col, start));
            }
            if c.is_ascii_digit() {
                return Ok(self.read_number(line, col, start));
            }
            if c == '"' || c == '\'' {
                return self.read_string(c, false, line, col, start);
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
                    line,
                    col,
                    ErrorMeta::default().with_code(codes::E001_UNEXPECTED_CHAR),
                ));
            }
            if c == '|' {
                self.advance();
                if self.current == Some('|') {
                    self.advance();
                    return Ok(Token::new(TokenType::Or, "||", line, col, start));
                }
                return Ok(Token::new(TokenType::Union, "|", line, col, start));
            }
            let (kind, lexeme) = match c {
                '?' => (TokenType::Optional, "?"),
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
                        line,
                        col,
                        ErrorMeta::default().with_code(codes::E001_UNEXPECTED_CHAR),
                    ));
                }
            };
            self.advance();
            return Ok(Token::new(kind, lexeme, line, col, start));
        }
        Ok(Token::new(
            TokenType::Eof,
            "",
            self.line,
            self.col,
            self.index,
        ))
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
        let (start_line, start_col) = (self.line, self.col);
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
            start_line,
            start_col,
            ErrorMeta::default().with_code(codes::E003_UNTERMINATED_COMMENT),
        ))
    }

    fn read_identifier(&mut self, start_line: usize, start_col: usize, start: usize) -> Token {
        let mut s = String::new();
        while self
            .current
            .map_or(false, |c| c.is_alphanumeric() || c == '_')
        {
            s.push(self.current.unwrap());
            self.advance();
        }
        let kind = keyword(&s).unwrap_or(TokenType::Identifier);
        Token::new(kind, s, start_line, start_col, start)
    }

    fn read_number(&mut self, start_line: usize, start_col: usize, start: usize) -> Token {
        let mut s = String::new();
        while self
            .current
            .map_or(false, |c| c.is_ascii_digit() || c == '.')
        {
            s.push(self.current.unwrap());
            self.advance();
        }
        Token::new(TokenType::Number, s, start_line, start_col, start)
    }

    fn read_string(
        &mut self,
        quote: char,
        escaped: bool,
        start_line: usize,
        start_col: usize,
        start: usize,
    ) -> Result<Token, SoplangError> {
        let delimiter: String = std::iter::repeat_n(quote, 3).collect();
        let triple = self.source[self.index..].starts_with(&delimiter);
        let width = if triple { 3 } else { 1 };
        for _ in 0..width {
            self.advance();
        }
        let mut value = String::new();
        while let Some(character) = self.current {
            if character == quote && (!triple || self.source[self.index..].starts_with(&delimiter))
            {
                for _ in 0..width {
                    self.advance();
                }
                let mut token = Token::new(TokenType::String, value, start_line, start_col, start);
                token.end = self.index;
                return Ok(token);
            }
            if escaped && character == '\\' {
                if let Some(character) = self.read_escape()? {
                    value.push(character);
                }
            } else if character == '\r' && self.peek() == Some('\n') {
                self.advance();
                self.advance();
                value.push('\n');
            } else {
                value.push(character);
                self.advance();
            }
        }
        Err(lexer_error_ex(
            "Qoraal aan la xidhin",
            start_line,
            start_col,
            ErrorMeta::default().with_code(codes::E002_UNTERMINATED_STRING),
        ))
    }

    fn read_formatted_string(
        &mut self,
        quote: char,
        line: usize,
        col: usize,
        start: usize,
    ) -> Result<Token, SoplangError> {
        if self.format_depth >= 64 {
            return Err(lexer_error_ex(
                "Qaybaha qoraalka aad bay isugu dhex jiraan.",
                line,
                col,
                ErrorMeta::default().with_code(codes::E005_INVALID_INTERPOLATION),
            ));
        }
        self.format_depth += 1;
        let result = self.read_formatted_inner(quote, line, col, start);
        self.format_depth -= 1;
        result
    }

    fn read_formatted_inner(
        &mut self,
        quote: char,
        line: usize,
        col: usize,
        start: usize,
    ) -> Result<Token, SoplangError> {
        use super::token::FormatPart;
        let delimiter: String = std::iter::repeat_n(quote, 3).collect();
        let triple = self.source[self.index..].starts_with(&delimiter);
        let width = if triple { 3 } else { 1 };
        for _ in 0..width {
            self.advance();
        }
        let content_start = self.index;
        let mut parts = Vec::new();
        let mut text = String::new();
        while let Some(character) = self.current {
            if character == quote && (!triple || self.source[self.index..].starts_with(&delimiter))
            {
                let content_end = self.index;
                for _ in 0..width {
                    self.advance();
                }
                if !text.is_empty() {
                    parts.push(FormatPart::Text(text));
                }
                let mut token = Token::new(
                    TokenType::FormattedString,
                    &self.source[content_start..content_end],
                    line,
                    col,
                    start,
                );
                token.end = self.index;
                token.format_parts = parts;
                return Ok(token);
            }
            if character == '\\' {
                if let Some(character) = self.read_escape()? {
                    text.push(character);
                }
            } else if character == '{' {
                if self.peek() == Some('{') {
                    self.advance();
                    self.advance();
                    text.push('{');
                } else {
                    if !text.is_empty() {
                        parts.push(FormatPart::Text(std::mem::take(&mut text)));
                    }
                    parts.push(self.read_format_field(quote, triple)?);
                }
            } else if character == '}' {
                if self.peek() != Some('}') {
                    return Err(lexer_error_ex(
                        "Calaamadda '}' ee qoraalka ma laha '{' u dhiganta.",
                        self.line,
                        self.col,
                        ErrorMeta::default().with_code(codes::E005_INVALID_INTERPOLATION),
                    ));
                }
                self.advance();
                self.advance();
                text.push('}');
            } else if character == '\r' && self.peek() == Some('\n') {
                self.advance();
                self.advance();
                text.push('\n');
            } else {
                text.push(character);
                self.advance();
            }
        }
        Err(lexer_error_ex(
            "Qoraal aan la xidhin",
            line,
            col,
            ErrorMeta::default().with_code(codes::E002_UNTERMINATED_STRING),
        ))
    }

    fn read_format_field(
        &mut self,
        outer_quote: char,
        triple: bool,
    ) -> Result<super::token::FormatPart, SoplangError> {
        use super::token::FormatPart;
        let (line, col) = (self.line, self.col);
        let unclosed = || {
            lexer_error_ex(
                "Qaybta qiimaha lagu gelinayo qoraalka lama xidhin.",
                line,
                col,
                ErrorMeta::default().with_code(codes::E005_INVALID_INTERPOLATION),
            )
        };
        self.advance(); // {
        let mut tokens: Vec<Token> = Vec::new();
        let mut braces = 0usize;
        let mut parens = 0usize;
        let mut brackets = 0usize;
        loop {
            loop {
                self.skip_whitespace();
                if self.current == Some('/') && self.skip_comment()? {
                    continue;
                }
                break;
            }
            if self.current.is_none() {
                return Err(unclosed());
            }
            // A closing outer quote after a complete atom indicates a missing }.
            let atom = tokens.last().is_some_and(|token| {
                matches!(
                    token.kind,
                    TokenType::Identifier
                        | TokenType::Number
                        | TokenType::String
                        | TokenType::FormattedString
                        | TokenType::True
                        | TokenType::False
                        | TokenType::Null
                        | TokenType::RParen
                        | TokenType::RBracket
                        | TokenType::RBrace
                )
            });
            let delimiter: String =
                std::iter::repeat_n(outer_quote, if triple { 3 } else { 1 }).collect();
            if braces == 0
                && parens == 0
                && brackets == 0
                && atom
                && self.source[self.index..].starts_with(&delimiter)
            {
                return Err(unclosed());
            }
            let token = self.next_token()?;
            let close = token.kind == TokenType::RBrace && braces == 0;
            let spec =
                token.kind == TokenType::Colon && braces == 0 && parens == 0 && brackets == 0;
            if close || spec {
                if tokens.is_empty() {
                    return Err(lexer_error_ex(
                        "Waxaa loo baahan yahay weedh gudaha '{...}' ee qoraalka.",
                        line,
                        col,
                        ErrorMeta::default().with_code(codes::E005_INVALID_INTERPOLATION),
                    ));
                }
                let mut eof = Token::new(TokenType::Eof, "", token.line, token.col, token.start);
                eof.end = token.start;
                tokens.push(eof);
                let mut format = String::new();
                let mut format_token = None;
                if spec {
                    let (format_start, format_line, format_col) = (self.index, self.line, self.col);
                    while let Some(character) = self.current {
                        if character == '}' {
                            break;
                        }
                        if character == '{' {
                            return Err(lexer_error_ex(
                                "Qaabka qoraalku ma aqbalo '{' gudaha meel qiime lagu gelinayo.",
                                self.line,
                                self.col,
                                ErrorMeta::default().with_code(codes::E005_INVALID_INTERPOLATION),
                            ));
                        }
                        format.push(character);
                        self.advance();
                    }
                    if self.current != Some('}') {
                        return Err(unclosed());
                    }
                    let mut position = Token::new(
                        TokenType::Identifier,
                        &format,
                        format_line,
                        format_col,
                        format_start,
                    );
                    position.end = self.index;
                    format_token = Some(position);
                    self.advance();
                }
                return Ok(FormatPart::Field {
                    tokens,
                    format,
                    format_token,
                });
            }
            match token.kind {
                TokenType::LBrace => braces += 1,
                TokenType::RBrace => braces = braces.saturating_sub(1),
                TokenType::LParen => parens += 1,
                TokenType::RParen => parens = parens.saturating_sub(1),
                TokenType::LBracket => brackets += 1,
                TokenType::RBracket => brackets = brackets.saturating_sub(1),
                TokenType::Eof => return Err(unclosed()),
                _ => (),
            }
            tokens.push(token);
        }
    }

    fn read_escape(&mut self) -> Result<Option<char>, SoplangError> {
        let (line, col) = (self.line, self.col);
        self.advance();
        let escaped = match self.current {
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            Some('0') => '\0',
            Some('\\') => '\\',
            Some('\'') => '\'',
            Some('"') => '"',
            Some('\n') => {
                self.advance();
                return Ok(None);
            }
            Some('\r') => {
                self.advance();
                if self.current == Some('\n') {
                    self.advance();
                }
                return Ok(None);
            }
            Some('u') => return self.read_unicode_escape(line, col).map(Some),
            Some(character) => {
                return Err(lexer_error_ex(
                    format!("Xarafka ka dambeeya '\\' lama aqoonsan: {}.", character),
                    line,
                    col,
                    ErrorMeta::default().with_code(codes::E004_INVALID_ESCAPE),
                ))
            }
            None => {
                return Err(lexer_error_ex(
                    "Xaraf waa inuu ka dambeeyaa calaamadda '\\'.",
                    line,
                    col,
                    ErrorMeta::default().with_code(codes::E004_INVALID_ESCAPE),
                ))
            }
        };
        self.advance();
        Ok(Some(escaped))
    }

    fn read_unicode_escape(&mut self, line: usize, col: usize) -> Result<char, SoplangError> {
        let invalid = || {
            lexer_error_ex(
                "Koodhka Unicode ee qoraalku ma saxna.",
                line,
                col,
                ErrorMeta::default().with_code(codes::E004_INVALID_ESCAPE),
            )
        };
        self.advance(); // u
        if self.current != Some('{') {
            return Err(invalid());
        }
        self.advance();
        let mut digits = String::new();
        while self
            .current
            .is_some_and(|character| character.is_ascii_hexdigit())
        {
            digits.push(self.current.unwrap());
            if digits.len() > 6 {
                return Err(invalid());
            }
            self.advance();
        }
        if self.current != Some('}') || digits.is_empty() {
            return Err(invalid());
        }
        let character = u32::from_str_radix(&digits, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or_else(invalid)?;
        self.advance();
        Ok(character)
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
