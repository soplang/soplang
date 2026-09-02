//! Resolve `keen` imports by loading and flattening imported modules at compile time.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{codes, import_error_ex, ErrorMeta, SoplangError};
use crate::frontend::ast::Stmt;
use crate::frontend::token::TokenType;
use crate::frontend::{Lexer, Parser};

/// Inline `keen` imports into one source string, in program order.
///
/// The result has no `keen` statements. Standalone builds embed this text and
/// run it without the original file path.
pub fn bundle_source(source: &str, source_path: Option<&Path>) -> Result<String, SoplangError> {
    let mut seen = HashSet::new();
    if let Some(path) = source_path {
        if let Ok(canon) = path.canonicalize() {
            seen.insert(canon);
        }
    }
    bundle_one(source, source_path, &mut seen)
}

fn bundle_one(
    source: &str,
    source_path: Option<&Path>,
    seen: &mut HashSet<PathBuf>,
) -> Result<String, SoplangError> {
    let tokens = Lexer::new(source).tokenize()?;
    let base_dir = source_path
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut out = String::new();
    let mut cursor = 0usize;
    let mut i = 0usize;
    while i < tokens.len() {
        if tokens[i].kind != TokenType::Keen {
            i += 1;
            continue;
        }
        let path_tok = tokens.get(i + 1).ok_or_else(|| {
            import_error_ex(
                "keen wuxuu u baahan yahay magaca faylka",
                tokens[i].line,
                tokens[i].col,
                ErrorMeta::default().with_code(codes::E021_UNDECLARED),
            )
        })?;
        if path_tok.kind != TokenType::String {
            return Err(import_error_ex(
                "keen wuxuu u baahan yahay magaca faylka",
                path_tok.line,
                path_tok.col,
                ErrorMeta::default().with_code(codes::E021_UNDECLARED),
            ));
        }

        let start = tokens[i].start;
        if start > cursor {
            out.push_str(&source[cursor..start]);
        }
        let import_end = tokens.get(i + 2).map(|t| t.start).unwrap_or(source.len());
        cursor = import_end;
        i += 2;

        let import_path = resolve_import_path(&base_dir, &path_tok.lexeme)?;
        let canon = import_path.canonicalize().map_err(|_| {
            import_error_ex(
                format!("Faylka '{}' ma helin", path_tok.lexeme),
                path_tok.line,
                path_tok.col,
                ErrorMeta::default()
                    .with_code(codes::E021_UNDECLARED)
                    .with_hint("Hubi in magaca faylka iyo jidka ay sax yihiin."),
            )
        })?;
        if !seen.insert(canon) {
            continue;
        }
        let imported = fs::read_to_string(&import_path).map_err(|_| {
            import_error_ex(
                format!("Faylka '{}' ma helin", path_tok.lexeme),
                path_tok.line,
                path_tok.col,
                ErrorMeta::default()
                    .with_code(codes::E021_UNDECLARED)
                    .with_hint("Hubi in faylka uu jiro oo akhriska loo ogol yahay."),
            )
        })?;
        let bundled = bundle_one(&imported, Some(&import_path), seen)?;
        out.push_str(&bundled);
        if !bundled.ends_with('\n') {
            out.push('\n');
        }
    }
    if cursor < source.len() {
        out.push_str(&source[cursor..]);
    }
    Ok(out)
}

/// Expand `Stmt::Import` nodes into the imported module's top-level statements.
pub fn resolve_imports(
    stmts: Vec<Stmt>,
    source_path: Option<&Path>,
) -> Result<Vec<Stmt>, SoplangError> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    if let Some(path) = source_path {
        if let Ok(canon) = path.canonicalize() {
            seen.insert(canon);
        }
    }
    flatten_imports(stmts, source_path, &mut out, &mut seen)?;
    Ok(out)
}

fn flatten_imports(
    stmts: Vec<Stmt>,
    source_path: Option<&Path>,
    out: &mut Vec<Stmt>,
    seen: &mut HashSet<PathBuf>,
) -> Result<(), SoplangError> {
    let base_dir = source_path
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    for stmt in stmts {
        match stmt {
            Stmt::Import(path) => {
                let import_path = resolve_import_path(&base_dir, &path)?;
                let canon = import_path.canonicalize().map_err(|_| {
                    import_error_ex(
                        format!("Faylka '{}' ma helin", path),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E021_UNDECLARED)
                            .with_hint("Hubi in magaca faylka iyo jidka ay sax yihiin."),
                    )
                })?;
                if !seen.insert(canon) {
                    continue;
                }
                let source = fs::read_to_string(&import_path).map_err(|_| {
                    import_error_ex(
                        format!("Faylka '{}' ma helin", path),
                        0,
                        0,
                        ErrorMeta::default()
                            .with_code(codes::E021_UNDECLARED)
                            .with_hint("Hubi in faylka uu jiro oo akhriska loo ogol yahay."),
                    )
                })?;
                let tokens = Lexer::new(&source).tokenize()?;
                let imported = Parser::new(tokens).parse()?;
                flatten_imports(imported, Some(&import_path), out, seen)?;
            }
            other => out.push(other),
        }
    }
    Ok(())
}

fn resolve_import_path(base_dir: &Path, path: &str) -> Result<PathBuf, SoplangError> {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        Ok(candidate)
    } else {
        Ok(base_dir.join(candidate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_inlines_keen_and_keeps_following_code() {
        let path = Path::new("examples/16_import.sop");
        let source = fs::read_to_string(path).unwrap();
        let bundled = bundle_source(&source, Some(path)).unwrap();
        assert!(!bundled.contains("keen"));
        let laba = bundled.find("hawl laba").expect("imported function");
        let call = bundled.find("laba() + afar()").expect("caller");
        assert!(laba < call);
    }
}
