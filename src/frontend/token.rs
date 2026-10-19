//! Token types and Token struct for the Soplang lexer.
//! Token types and keywords for Soplang.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Keywords — control flow & declarations
    Door,      // door
    Madoor,    // madoor (const)
    Hawl,      // hawl (function)
    Celi,      // celi (return)
    Qor,       // qor (print)
    Gelin,     // gelin (input)
    Haddii,    // haddii (if)
    HaddiiKale,// haddii_kale (else if)
    Ugudambeyn,// ugudambeyn (else)
    Dooro,     // dooro (switch)
    Xaalad,    // xaalad (case)
    Kuceli,    // kuceli (for)
    Intay,     // intay (while)
    Jooji,     // jooji (break)
    Soco,      // soco (continue)
    Fasax,     // fasax (try)
    Qabo,      // qabo (catch)
    Keen,      // keen (import)
    Qaab,      // qaab (class)
    Dhaxal,    // dhaxal (extends)
    Cusub,     // cusub (new)
    Nafta,     // nafta (self/this)
    Waalid,    // waalid (parent method)
    // Static type keywords
    Abn,       // abn (integer)
    Jajab,     // jajab (float)
    Qoraal,    // qoraal (string)
    Bool,      // bool
    Teed,      // teed (list)
    Walax,     // walax (object)
    // Literals
    True,      // run
    False,     // been
    Null,      // maran (null compatibility spelling)
    Identifier,
    Number,
    String,
    FormattedString,
    // Operators
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Modulo,    // %
    EqEq,      // ==
    NotEq,     // !=
    Greater,   // >
    Less,      // <
    GreaterEq, // >=
    LessEq,    // <=
    And,       // &&
    Or,        // ||
    Not,       // !
    Assign,    // =
    // Structural
    Comma,     // ,
    Colon,     // :
    Semicolon, // ;
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
    Dot,       // .
    Eof,
}

#[derive(Debug, Clone)]
pub enum FormatPart {
    Text(String),
    Field {
        tokens: Vec<Token>,
        format: String,
        format_token: Option<Token>,
    },
}

/// A single token with source location.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind:   TokenType,
    pub lexeme: String,
    pub line:   usize,
    pub col:    usize,
    /// Byte offset of this token in the source.
    pub start:  usize,
    /// Exclusive byte offset, including prefixes and string delimiters.
    pub end: usize,
    pub format_parts: Vec<FormatPart>,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: impl Into<String>, line: usize, col: usize, start: usize) -> Self {
        let lexeme = lexeme.into();
        let end = start + lexeme.len() + usize::from(kind == TokenType::String) * 2;
        Self { kind, lexeme, line, col, start, end, format_parts: Vec::new() }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lexeme = if self.lexeme.is_empty() { String::new() } else { format!(" {:?}", self.lexeme) };
        write!(f, "Token({:?}{} line={} col={})", self.kind, lexeme, self.line, self.col)
    }
}
