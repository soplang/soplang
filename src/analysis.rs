//! Read-only editor analysis. Never lowers HIR or imports runtime code.
use crate::{
    error::{import_error_ex, ErrorMeta, SoplangError},
    frontend::{self, Stmt},
    semantic::{analyze_with_options, AnalyzeOptions, SymbolTable},
};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

pub mod index;

#[derive(Clone, Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    pub statements: Vec<Stmt>,
}
pub struct Analysis {
    pub files: Vec<SourceFile>,
    pub statements: Vec<Stmt>,
    pub symbols: Option<SymbolTable>,
    pub error: Option<SoplangError>,
}
#[derive(Clone, Copy)]
pub struct Limits {
    pub max_file_bytes: usize,
    pub max_files: usize,
    pub max_total_bytes: usize,
    pub max_depth: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_file_bytes: 1024 * 1024,
            max_files: 128,
            max_total_bytes: 8 * 1024 * 1024,
            max_depth: 32,
        }
    }
}
/// Lexically normalize new overlay paths, and canonicalize existing paths.
pub fn source_identity(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => (),
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
/// `load` must return overlays before disk. Cancellation is checked at each import.
/// Analysis always uses file semantics, including for untitled editing.
pub fn check_source(
    source: &str,
    path: &Path,
    strict: bool,
    limits: Limits,
    load: &dyn Fn(&Path) -> Result<String, String>,
    cancelled: &dyn Fn() -> bool,
) -> Analysis {
    struct Loader<'a> {
        files: Vec<SourceFile>,
        seen: HashSet<PathBuf>,
        bytes: usize,
        limits: Limits,
        load: &'a dyn Fn(&Path) -> Result<String, String>,
        cancelled: &'a dyn Fn() -> bool,
    }
    impl Loader<'_> {
        fn one(
            &mut self,
            source: &str,
            path: &Path,
            depth: usize,
        ) -> Result<Vec<Stmt>, SoplangError> {
            let path = source_identity(path);
            if !self.seen.insert(path.clone()) {
                return Ok(vec![]);
            }
            self.bytes += source.len();
            if (self.cancelled)()
                || source.len() > self.limits.max_file_bytes
                || self.bytes > self.limits.max_total_bytes
                || self.seen.len() > self.limits.max_files
                || depth > self.limits.max_depth
            {
                return Err(import_error_ex(
                    "Analysis limit reached or request cancelled",
                    1,
                    1,
                    ErrorMeta::default()
                        .with_code("EDITOR_LIMIT")
                        .with_file(path),
                ));
            }
            let tokens = frontend::Lexer::new(source).tokenize();
            if let Ok(tokens) = &tokens {
                let mut nesting = 0usize;
                let mut operators = 0usize;
                for token in tokens {
                    match token.kind {
                        frontend::TokenType::LBrace
                        | frontend::TokenType::LBracket
                        | frontend::TokenType::LParen => nesting += 1,
                        frontend::TokenType::RBrace
                        | frontend::TokenType::RBracket
                        | frontend::TokenType::RParen => {
                            nesting = nesting.saturating_sub(1);
                            operators = 0;
                        }
                        frontend::TokenType::Plus
                        | frontend::TokenType::Minus
                        | frontend::TokenType::Not
                        | frontend::TokenType::Star
                        | frontend::TokenType::Slash
                        | frontend::TokenType::And
                        | frontend::TokenType::Or => operators += 1,
                        frontend::TokenType::Assign
                        | frontend::TokenType::Comma
                        | frontend::TokenType::Semicolon => operators = 0,
                        _ => (),
                    }
                    if nesting > 128 || operators > 512 || tokens.len() > 100_000 {
                        return Err(import_error_ex(
                            "Source exceeds editor syntax complexity limit",
                            1,
                            1,
                            ErrorMeta::default()
                                .with_code("EDITOR_LIMIT")
                                .with_file(path),
                        ));
                    }
                }
            }
            let parsed = frontend::parse_source(source, Some(&path));
            let partial = match &parsed {
                Ok(stmts) => stmts.clone(),
                Err(_) => frontend::parse_partial(source, Some(&path)),
            };
            self.files.push(SourceFile {
                path: path.clone(),
                text: source.to_owned(),
                statements: partial,
            });
            let mut flattened = vec![];
            for stmt in parsed? {
                if let Stmt::Import(target) = stmt.unlocated() {
                    let target =
                        source_identity(&path.parent().unwrap_or(Path::new(".")).join(target));
                    let result = (self.load)(&target).map_err(|message| {
                        import_error_ex(message, 0, 0, ErrorMeta::default().with_code("E021"))
                    });
                    let result = result.and_then(|text| self.one(&text, &target, depth + 1));
                    flattened.extend(result.map_err(|e| {
                        if let Stmt::Located { span, .. } = &stmt {
                            span.attach(e)
                        } else {
                            e
                        }
                    })?);
                } else {
                    flattened.push(stmt);
                }
            }
            Ok(flattened)
        }
    }
    let mut loader = Loader {
        files: vec![],
        seen: HashSet::new(),
        bytes: 0,
        limits,
        load,
        cancelled,
    };
    let parsed = loader.one(source, path, 0);
    let (statements, symbols, error) = match parsed {
        Ok(stmts) => match analyze_with_options(
            &stmts,
            AnalyzeOptions {
                strict,
                repl: false,
            },
        ) {
            Ok(sym) => (stmts, Some(sym), None),
            Err(error) => (stmts, None, Some(error)),
        },
        Err(error) => (vec![], None, Some(error)),
    };
    Analysis {
        files: loader.files,
        statements,
        symbols,
        error,
    }
}
/// Read-only filesystem provider with unsaved entry/import overlays.
pub fn overlay_loader(overlays: &HashMap<PathBuf, String>, path: &Path) -> Result<String, String> {
    let path = source_identity(path);
    overlays.get(&path).cloned().map(Ok).unwrap_or_else(|| {
        std::fs::read_to_string(&path).map_err(|_| format!("Faylka '{}' ma helin", path.display()))
    })
}
