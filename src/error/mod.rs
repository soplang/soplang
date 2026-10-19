//! Soplang error types with Somali messages.
//! Structured errors with codes, spans, and hints.

mod external;
mod format;
pub use external::{
    external_error, file_read_error, input_error, io_error, io_reason, tool_start_error,
};

use std::fmt;
use std::path::PathBuf;

pub use format::format_error_with_source;

// ---------------------------------------------------------------------------
// Error codes (machine-readable, documented)
// ---------------------------------------------------------------------------

pub mod codes {
    pub const E001_UNEXPECTED_CHAR: &str = "E001";
    pub const E002_UNTERMINATED_STRING: &str = "E002";
    pub const E003_UNTERMINATED_COMMENT: &str = "E003";
    pub const E004_INVALID_ESCAPE: &str = "E004";
    pub const E005_INVALID_INTERPOLATION: &str = "E005";
    pub const E010_UNEXPECTED_TOKEN: &str = "E010";
    pub const E011_INVALID_NUMBER: &str = "E011";
    pub const E012_EXPECTED_NAME: &str = "E012";
    pub const E020_REDECLARED: &str = "E020";
    pub const E021_UNDECLARED: &str = "E021";
    pub const E022_TYPE_MISMATCH: &str = "E022";
    pub const E030_DIVISION_BY_ZERO: &str = "E030";
    pub const E031_INDEX_OUT_OF_BOUNDS: &str = "E031";
    pub const E032_BREAK_OUTSIDE_LOOP: &str = "E032";
    pub const E033_NOT_A_FUNCTION: &str = "E033";
    pub const E034_UNKNOWN_METHOD: &str = "E034";
    pub const E035_INVALID_HANDLE: &str = "E035";
    pub const E036_INVALID_FORMAT: &str = "E036";
    pub const E037_INVALID_ITERATION: &str = "E037";
}

/// A compiled function and its caller's source location.
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function: String,
    pub file: Option<PathBuf>,
    pub line: usize,
    pub col: usize,
}

/// Optional metadata for an error (code, span end, hint, file).
#[derive(Debug, Clone, Default)]
pub struct ErrorMeta {
    pub code: Option<&'static str>,
    pub user_code: Option<String>,
    pub cause: Option<Box<SoplangError>>,
    pub trace: Vec<StackFrame>,
    pub end_line: Option<usize>,
    pub end_col: Option<usize>,
    pub hint: Option<String>,
    pub file: Option<PathBuf>,
    pub source_line: Option<String>,
    /// Original OS/tool text; never part of a caught runtime message.
    pub technical_details: Option<String>,
}

impl ErrorMeta {
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.technical_details = Some(details.into());
        self
    }
    pub fn with_code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }
    pub fn with_span(mut self, end_line: usize, end_col: usize) -> Self {
        self.end_line = Some(end_line);
        self.end_col = Some(end_col);
        self
    }
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }
}

/// One source token, retained through compilation for precise diagnostics.
#[derive(Debug, Clone)]
pub struct SourceSpan {
    pub file: Option<PathBuf>,
    pub line: usize,
    pub col: usize,
    pub end_col: usize,
    pub source_line: String,
}

impl SourceSpan {
    pub fn attach(&self, mut error: SoplangError) -> SoplangError {
        let (line, col, meta) = match &mut error {
            SoplangError::Lexer {
                line, col, meta, ..
            }
            | SoplangError::Parser {
                line, col, meta, ..
            }
            | SoplangError::Runtime {
                line, col, meta, ..
            }
            | SoplangError::Type {
                line, col, meta, ..
            }
            | SoplangError::Import {
                line, col, meta, ..
            } => (line, col, meta),
        };
        if *line == 0 {
            *line = self.line;
            *col = self.col;
        }
        if meta.file.is_none() {
            meta.file = self.file.clone();
        }
        if *line == self.line && meta.source_line.is_none() {
            meta.source_line = Some(self.source_line.clone());
            if *col == self.col {
                meta.end_line = Some(self.line);
                meta.end_col = Some(self.end_col);
            }
        }
        error
    }
}

#[derive(Debug, Clone)]
pub enum SoplangError {
    Lexer {
        msg: String,
        line: usize,
        col: usize,
        meta: ErrorMeta,
    },
    Parser {
        msg: String,
        line: usize,
        col: usize,
        meta: ErrorMeta,
    },
    Runtime {
        msg: String,
        line: usize,
        col: usize,
        meta: ErrorMeta,
    },
    Type {
        msg: String,
        line: usize,
        col: usize,
        meta: ErrorMeta,
    },
    Import {
        msg: String,
        line: usize,
        col: usize,
        meta: ErrorMeta,
    },
}

impl fmt::Display for SoplangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, msg, line, col, code) = match self {
            SoplangError::Lexer {
                msg,
                line,
                col,
                meta,
            } => (
                "ereyayn",
                msg,
                line,
                col,
                meta.user_code.as_deref().or(meta.code),
            ),
            SoplangError::Parser {
                msg,
                line,
                col,
                meta,
            } => (
                "naxwe",
                msg,
                line,
                col,
                meta.user_code.as_deref().or(meta.code),
            ),
            SoplangError::Runtime {
                msg,
                line,
                col,
                meta,
            } => (
                "fulin",
                msg,
                line,
                col,
                meta.user_code.as_deref().or(meta.code),
            ),
            SoplangError::Type {
                msg,
                line,
                col,
                meta,
            } => (
                "nooc",
                msg,
                line,
                col,
                meta.user_code.as_deref().or(meta.code),
            ),
            SoplangError::Import {
                msg,
                line,
                col,
                meta,
            } => (
                "soo-dejin",
                msg,
                line,
                col,
                meta.user_code.as_deref().or(meta.code),
            ),
        };
        let code_str = code.map(|c| format!(" [{}]", c)).unwrap_or_default();
        write!(f, "Khalad {}: {}{}", kind, msg, code_str)?;
        if *line > 0 && *col > 0 {
            write!(f, " safka {}, tiirka {}", line, col)?;
        }
        Ok(())
    }
}

impl std::error::Error for SoplangError {}

impl SoplangError {
    pub fn message(&self) -> &str {
        match self {
            Self::Lexer { msg, .. }
            | Self::Parser { msg, .. }
            | Self::Runtime { msg, .. }
            | Self::Type { msg, .. }
            | Self::Import { msg, .. } => msg,
        }
    }
    pub fn location(&self) -> (usize, usize) {
        match self {
            SoplangError::Lexer { line, col, .. }
            | SoplangError::Parser { line, col, .. }
            | SoplangError::Runtime { line, col, .. }
            | SoplangError::Type { line, col, .. }
            | SoplangError::Import { line, col, .. } => (*line, *col),
        }
    }
    pub fn code(&self) -> Option<&str> {
        match self {
            SoplangError::Lexer { meta, .. }
            | SoplangError::Parser { meta, .. }
            | SoplangError::Runtime { meta, .. }
            | SoplangError::Type { meta, .. }
            | SoplangError::Import { meta, .. } => meta.user_code.as_deref().or(meta.code),
        }
    }
    pub fn hint(&self) -> Option<&str> {
        match self {
            SoplangError::Lexer { meta, .. }
            | SoplangError::Parser { meta, .. }
            | SoplangError::Runtime { meta, .. }
            | SoplangError::Type { meta, .. }
            | SoplangError::Import { meta, .. } => meta.hint.as_deref(),
        }
    }
    pub(crate) fn meta_mut(&mut self) -> &mut ErrorMeta {
        match self {
            Self::Lexer { meta, .. }
            | Self::Parser { meta, .. }
            | Self::Runtime { meta, .. }
            | Self::Type { meta, .. }
            | Self::Import { meta, .. } => meta,
        }
    }

    pub fn meta(&self) -> &ErrorMeta {
        match self {
            SoplangError::Lexer { meta, .. }
            | SoplangError::Parser { meta, .. }
            | SoplangError::Runtime { meta, .. }
            | SoplangError::Type { meta, .. }
            | SoplangError::Import { meta, .. } => meta,
        }
    }
}

pub fn lexer_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Lexer {
        msg: msg.into(),
        line,
        col,
        meta: ErrorMeta::default(),
    }
}
pub fn lexer_error_ex(
    msg: impl Into<String>,
    line: usize,
    col: usize,
    meta: ErrorMeta,
) -> SoplangError {
    SoplangError::Lexer {
        msg: msg.into(),
        line,
        col,
        meta,
    }
}
pub fn parser_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Parser {
        msg: msg.into(),
        line,
        col,
        meta: ErrorMeta::default(),
    }
}
pub fn parser_error_ex(
    msg: impl Into<String>,
    line: usize,
    col: usize,
    meta: ErrorMeta,
) -> SoplangError {
    SoplangError::Parser {
        msg: msg.into(),
        line,
        col,
        meta,
    }
}
pub fn runtime_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Runtime {
        msg: msg.into(),
        line,
        col,
        meta: ErrorMeta::default(),
    }
}
pub fn runtime_error_ex(
    msg: impl Into<String>,
    line: usize,
    col: usize,
    meta: ErrorMeta,
) -> SoplangError {
    SoplangError::Runtime {
        msg: msg.into(),
        line,
        col,
        meta,
    }
}
pub fn type_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Type {
        msg: msg.into(),
        line,
        col,
        meta: ErrorMeta::default(),
    }
}
pub fn type_error_ex(
    msg: impl Into<String>,
    line: usize,
    col: usize,
    meta: ErrorMeta,
) -> SoplangError {
    SoplangError::Type {
        msg: msg.into(),
        line,
        col,
        meta,
    }
}
pub fn import_error(msg: impl Into<String>, line: usize, col: usize) -> SoplangError {
    SoplangError::Import {
        msg: msg.into(),
        line,
        col,
        meta: ErrorMeta::default(),
    }
}
pub fn import_error_ex(
    msg: impl Into<String>,
    line: usize,
    col: usize,
    meta: ErrorMeta,
) -> SoplangError {
    SoplangError::Import {
        msg: msg.into(),
        line,
        col,
        meta,
    }
}
