//! Read-only editor analysis. Never lowers HIR or imports runtime code.
use crate::{
    error::{import_error_ex, ErrorMeta, SoplangError},
    frontend::{self, Stmt},
    semantic::{analyze_with_options, AnalyzeOptions, SymbolTable},
};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

pub mod index;

#[derive(Clone, Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    pub statements: Vec<Stmt>,
    pub imports: Vec<(frontend::ast::ImportSpec, PathBuf)>,
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
    let path = source_identity(path);
    let mut sources = EditorSources {
        files: Vec::new(),
        cached: HashMap::new(),
        bytes: 0,
        limits,
        load,
        cancelled,
    };
    let result = frontend::imports::ImportSources::observe(&mut sources, &path, source, 0)
        .and_then(|_| {
            match frontend::parse_source(source, Some(&path)) {
                Ok(statements) => frontend::imports::ImportResolver::default().resolve_using(
                    statements,
                    Some(&path),
                    &mut sources,
                ),
                Err(error) => {
                    // An unfinished expression must not hide earlier imports
                    // from completion. Retain the original syntax diagnostic;
                    // only resolve the already recovered, complete statements.
                    let partial = sources
                        .files
                        .iter()
                        .find(|file| file.path == path)
                        .map(|file| file.statements.clone())
                        .unwrap_or_default();
                    let _ = frontend::imports::ImportResolver::default().resolve_using(
                        partial,
                        Some(&path),
                        &mut sources,
                    );
                    Err(error)
                }
            }
        });
    let (statements, symbols, error) = match result {
        Ok(statements) => match analyze_with_options(
            &statements,
            AnalyzeOptions {
                strict,
                repl: false,
            },
        ) {
            Ok(symbols) => (statements, Some(symbols), None),
            Err(error) => (statements, None, Some(error)),
        },
        Err(error) => (Vec::new(), None, Some(error)),
    };
    Analysis {
        files: sources.files,
        statements,
        symbols,
        error,
    }
}

struct EditorSources<'a> {
    files: Vec<SourceFile>,
    cached: HashMap<PathBuf, Result<String, String>>,
    bytes: usize,
    limits: Limits,
    load: &'a dyn Fn(&Path) -> Result<String, String>,
    cancelled: &'a dyn Fn() -> bool,
}

impl EditorSources<'_> {
    fn contents(&mut self, path: &Path) -> Result<String, String> {
        self.cached
            .entry(source_identity(path))
            .or_insert_with(|| (self.load)(path))
            .clone()
    }

    fn limited(&self, path: &Path) -> SoplangError {
        import_error_ex(
            "Falanqayntu waxay gaadhay xadka ama waa la joojiyey.",
            1,
            1,
            ErrorMeta::default()
                .with_code("EDITOR_LIMIT")
                .with_file(path.to_path_buf()),
        )
    }
}

impl frontend::imports::ImportSources for EditorSources<'_> {
    fn read_only(&self) -> bool {
        true
    }

    fn read(&mut self, path: &Path) -> Result<String, SoplangError> {
        if (self.cancelled)() {
            return Err(self.limited(path));
        }
        self.contents(path).map_err(|message| {
            import_error_ex(
                format!("Faylka '{}' lama akhriyi karo.", path.display()),
                0,
                0,
                ErrorMeta::default().with_code("E021").with_details(message),
            )
        })
    }

    fn exists(&mut self, path: &Path) -> bool {
        if (self.cancelled)() {
            return false;
        }
        path.is_file() || self.contents(path).is_ok()
    }

    fn observe(&mut self, path: &Path, source: &str, depth: usize) -> Result<(), SoplangError> {
        if (self.cancelled)() {
            return Err(self.limited(path));
        }
        let path = source_identity(path);
        if self.files.iter().any(|file| file.path == path) {
            return Ok(());
        }
        self.bytes = self.bytes.saturating_add(source.len());
        if source.len() > self.limits.max_file_bytes
            || self.bytes > self.limits.max_total_bytes
            || self.files.len() >= self.limits.max_files
            || depth > self.limits.max_depth
        {
            return Err(self.limited(&path));
        }
        if let Ok(tokens) = frontend::Lexer::new(source).tokenize() {
            fn bounded(tokens: &[frontend::Token], total: &mut usize) -> bool {
                use frontend::TokenType as K;
                let (mut nesting, mut operators) = (0usize, 0usize);
                for token in tokens {
                    *total += 1;
                    match token.kind {
                        K::LBrace | K::LBracket | K::LParen => nesting += 1,
                        K::RBrace | K::RBracket | K::RParen => {
                            nesting = nesting.saturating_sub(1);
                            operators = 0;
                        }
                        K::Plus | K::Minus | K::Not | K::Star | K::Slash | K::And | K::Or => {
                            operators += 1
                        }
                        K::Assign | K::Comma | K::Semicolon => operators = 0,
                        _ => {}
                    }
                    if nesting > 128 || operators > 512 || *total > 100_000 {
                        return false;
                    }
                    for part in &token.format_parts {
                        if let frontend::token::FormatPart::Field { tokens, .. } = part {
                            if !bounded(tokens, total) {
                                return false;
                            }
                        }
                    }
                }
                true
            }
            if !bounded(&tokens, &mut 0) {
                return Err(self.limited(&path));
            }
        }
        let statements = frontend::parse_source(source, Some(&path))
            .unwrap_or_else(|_| frontend::parse_partial(source, Some(&path)));
        self.files.push(SourceFile {
            path,
            text: source.to_string(),
            statements,
            imports: Vec::new(),
        });
        Ok(())
    }

    fn imported(&mut self, from: Option<&Path>, target: &Path, spec: &frontend::ast::ImportSpec) {
        if let Some(from) = from {
            let from = source_identity(from);
            if let Some(file) = self.files.iter_mut().find(|file| file.path == from) {
                file.imports.push((spec.clone(), source_identity(target)));
            }
        }
    }
}

/// Read-only filesystem provider with unsaved entry/import overlays.
pub fn overlay_loader(overlays: &HashMap<PathBuf, String>, path: &Path) -> Result<String, String> {
    use std::io::Read;
    let path = source_identity(path);
    let max = Limits::default().max_file_bytes;
    if let Some(text) = overlays.get(&path) {
        return if text.len() <= max {
            Ok(text.clone())
        } else {
            Err("Faylku wuxuu dhaafay xadka qoraalka ee tifaftiraha.".into())
        };
    }
    let meta = std::fs::metadata(&path).map_err(|error| {
        crate::error::file_read_error(&path, &error)
            .message()
            .to_string()
    })?;
    if !meta.is_file() {
        return Err("Isha barnaamijku waa inay noqotaa fayl caadi ah.".into());
    }
    if meta.len() > max as u64 {
        return Err("Faylku wuxuu dhaafay xadka qoraalka ee tifaftiraha.".into());
    }
    let file = std::fs::File::open(&path).map_err(|error| {
        crate::error::file_read_error(&path, &error)
            .message()
            .to_string()
    })?;
    let mut source = String::new();
    file.take(max as u64 + 1)
        .read_to_string(&mut source)
        .map_err(|error| {
            crate::error::file_read_error(&path, &error)
                .message()
                .to_string()
        })?;
    if source.len() > max {
        return Err("Faylku wuxuu dhaafay xadka qoraalka ee tifaftiraha.".into());
    }
    Ok(source)
}
