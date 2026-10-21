//! Resolve `keen` imports by loading and flattening imported modules at compile time.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{codes, file_read_error, import_error_ex, ErrorMeta, SoplangError};
use crate::frontend::ast::{Expr, Stmt, TypeAnnotation};
use crate::frontend::token::TokenType;
use crate::frontend::Lexer;

/// Inline `keen` imports into one source string, in program order.
///
/// The result has no `keen` statements. Standalone builds compile this text
/// without needing the original import files at runtime.
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
        if tokens.get(i + 2).is_some_and(|token| token.kind == TokenType::Identifier && token.lexeme == "sida")
            && tokens.get(i + 3).is_some_and(|token| token.kind == TokenType::Identifier)
        {
            let end = tokens.get(i + 4).map(|token| token.start).unwrap_or(source.len());
            if start > cursor {
                out.push_str(&source[cursor..start]);
            }
            out.push_str(&source[start..end]);
            cursor = end;
            i += 4;
            continue;
        }
        if start > cursor {
            out.push_str(&source[cursor..start]);
        }
        let import_end = tokens.get(i + 2).map(|t| t.start).unwrap_or(source.len());
        cursor = import_end;
        i += 2;

        let import_path = resolve_import_path(&base_dir, &path_tok.lexeme)?;
        let canon = import_path.canonicalize().map_err(|e| {
            let error = file_read_error(Path::new(&path_tok.lexeme), &e);
            import_error_ex(
                error.message(),
                path_tok.line,
                path_tok.col,
                error
                    .meta()
                    .clone()
                    .with_code(codes::E021_UNDECLARED)
                    .with_hint("Hubi in magaca faylka iyo jidka ay sax yihiin."),
            )
        })?;
        if !seen.insert(canon) {
            continue;
        }
        let imported = fs::read_to_string(&import_path).map_err(|e| {
            let error = file_read_error(Path::new(&path_tok.lexeme), &e);
            import_error_ex(
                error.message(),
                path_tok.line,
                path_tok.col,
                error
                    .meta()
                    .clone()
                    .with_code(codes::E021_UNDECLARED)
                    .with_hint("Hubi in faylka uu jiro oo akhriska loo oggol yahay."),
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
            Stmt::Located { span, stmt } if matches!(stmt.unlocated(), Stmt::Import(_)) => {
                flatten_imports(vec![*stmt], source_path, out, seen).map_err(|e| span.attach(e))?;
            }
            Stmt::Import(import) => {
                let import_path = resolve_import_path(&base_dir, &import.path)?;
                let canon = import_path.canonicalize().map_err(|e| {
                    let error = file_read_error(Path::new(&import.path), &e);
                    import_error_ex(
                        error.message(),
                        0,
                        0,
                        error
                            .meta()
                            .clone()
                            .with_code(codes::E021_UNDECLARED)
                            .with_hint("Hubi in magaca faylka iyo jidka ay sax yihiin."),
                    )
                })?;
                if !seen.insert(canon) {
                    if import.alias.is_some() {
                        return Err(import_error_ex(
                            "Keen-ku waa wareeg, ama faylkan horay ayaa loo keenay.",
                            0,
                            0,
                            ErrorMeta::default().with_code(codes::E021_UNDECLARED),
                        ));
                    }
                    continue;
                }
                let source = fs::read_to_string(&import_path).map_err(|e| {
                    let error = file_read_error(Path::new(&import.path), &e);
                    import_error_ex(
                        error.message(),
                        0,
                        0,
                        error
                            .meta()
                            .clone()
                            .with_code(codes::E021_UNDECLARED)
                            .with_hint("Hubi in faylka uu jiro oo akhriska loo oggol yahay."),
                    )
                })?;
                let imported = super::parse_source(&source, Some(&import_path))?;
                if let Some(alias) = import.alias {
                    let mut body = Vec::new();
                    flatten_imports(imported, Some(&import_path), &mut body, seen)?;
                    let public = export_names(&body)?;
                    let renamed = mangled_bindings(&alias, &body);
                    rename_stmts(&mut body, &renamed);
                    body.push(Stmt::Return(Some(Expr::Object(
                        public
                            .into_iter()
                            .map(|name| {
                                let value = renamed.get(&name).cloned().unwrap_or_else(|| name.clone());
                                (name, Expr::Identifier(value))
                            })
                            .collect(),
                    ))));
                    out.push(Stmt::VarDecl {
                        name: alias,
                        type_ann: TypeAnnotation::Dynamic,
                        is_const: true,
                        value: Expr::Invoke {
                            callee: Box::new(Expr::AnonymousFunc {
                                params: Vec::new(),
                                return_ann: TypeAnnotation::Dynamic,
                                body,
                            }),
                            args: Vec::new(),
                        },
                        line: 0,
                        col: 0,
                    });
                } else {
                    flatten_imports(imported, Some(&import_path), out, seen)?;
                }
            }
            other => out.push(other),
        }
    }
    Ok(())
}

fn export_names(stmts: &[Stmt]) -> Result<Vec<String>, SoplangError> {
    let mut defined = HashSet::new();
    let mut explicit = Vec::new();
    for stmt in stmts {
        match stmt.unlocated() {
            Stmt::Export(names) => explicit.extend(names.iter().cloned()),
            Stmt::FuncDef { name, .. } | Stmt::VarDecl { name, .. } => {
                defined.insert(name.clone());
            }
            _ => {}
        }
    }
    if explicit.is_empty() {
        let mut names: Vec<String> = defined
            .into_iter()
            .filter(|name| !name.starts_with('_'))
            .collect();
        names.sort();
        return Ok(names);
    }
    let mut names = Vec::new();
    for name in explicit {
        if !defined.contains(&name) {
            return Err(import_error_ex(
                format!("'{name}' lama soo saari karo sababtoo ah lama qeexin."),
                0,
                0,
                ErrorMeta::default().with_code(codes::E021_UNDECLARED),
            ));
        }
        if !names.contains(&name) {
            names.push(name);
        }
    }
    Ok(names)
}

fn mangled_bindings(alias: &str, stmts: &[Stmt]) -> HashMap<String, String> {
    let mut names = HashMap::new();
    for stmt in stmts {
        if let Stmt::FuncDef { name, .. } | Stmt::ClassDef { name, .. } = stmt.unlocated() {
            names.insert(name.clone(), format!("__soplang_{alias}_{name}"));
        }
    }
    names
}

fn rename_stmts(stmts: &mut [Stmt], names: &HashMap<String, String>) {
    for stmt in stmts {
        rename_stmt(stmt, names);
    }
}

fn rename_stmt(stmt: &mut Stmt, names: &HashMap<String, String>) {
    match stmt {
        Stmt::Located { stmt, .. } => rename_stmt(stmt, names),
        Stmt::VarDecl { value, .. } => rename_expr(value, names),
        Stmt::Assign { target, value, .. } => {
            rename_expr(target, names);
            rename_expr(value, names);
        }
        Stmt::FuncDef { name, body, .. } => {
            if let Some(renamed) = names.get(name) {
                *name = renamed.clone();
            }
            rename_stmts(body, names);
        }
        Stmt::ClassDef { name, parent, body } => {
            if let Some(renamed) = names.get(name) {
                *name = renamed.clone();
            }
            if let Some(parent) = parent {
                if let Some(renamed) = names.get(parent) {
                    *parent = renamed.clone();
                }
            }
            rename_stmts(body, names);
        }
        Stmt::If { cond, then_body, elseifs, else_body } => {
            rename_expr(cond, names);
            rename_stmts(then_body, names);
            for (cond, body) in elseifs {
                rename_expr(cond, names);
                rename_stmts(body, names);
            }
            if let Some(body) = else_body {
                rename_stmts(body, names);
            }
        }
        Stmt::Switch { expr, cases, default } => {
            rename_expr(expr, names);
            for (value, body) in cases {
                rename_expr(value, names);
                rename_stmts(body, names);
            }
            if let Some(body) = default {
                rename_stmts(body, names);
            }
        }
        Stmt::For { start, end, step, body, .. } => {
            rename_expr(start, names);
            rename_expr(end, names);
            if let Some(step) = step {
                rename_expr(step, names);
            }
            rename_stmts(body, names);
        }
        Stmt::ForEach { iterable, body, .. } => {
            rename_expr(iterable, names);
            rename_stmts(body, names);
        }
        Stmt::While { cond, body } => {
            rename_expr(cond, names);
            rename_stmts(body, names);
        }
        Stmt::Return(value) => {
            if let Some(value) = value {
                rename_expr(value, names);
            }
        }
        Stmt::Throw(value) | Stmt::Expr(value) => rename_expr(value, names),
        Stmt::TryCatch { try_body, catch_body, finally_body, .. } => {
            rename_stmts(try_body, names);
            rename_stmts(catch_body, names);
            rename_stmts(finally_body, names);
        }
        Stmt::Block(body) => rename_stmts(body, names),
        Stmt::Break | Stmt::Continue | Stmt::Import(_) | Stmt::Export(_) => {}
    }
}

fn rename_expr(expr: &mut Expr, names: &HashMap<String, String>) {
    match expr {
        Expr::Located { expr, .. } => rename_expr(expr, names),
        Expr::Identifier(name) | Expr::Call { name, .. } => {
            if let Some(renamed) = names.get(name) {
                *name = renamed.clone();
            }
            if let Expr::Call { args, .. } = expr {
                for arg in args {
                    rename_expr(arg, names);
                }
            }
        }
        Expr::BinaryOp { left, right, .. } => {
            rename_expr(left, names);
            rename_expr(right, names);
        }
        Expr::UnaryOp { expr, .. } => rename_expr(expr, names),
        Expr::Invoke { callee, args } | Expr::MethodCall { obj: callee, args, .. } => {
            rename_expr(callee, names);
            for arg in args {
                rename_expr(arg, names);
            }
        }
        Expr::AnonymousFunc { body, .. } => rename_stmts(body, names),
        Expr::Index { obj, idx } => {
            rename_expr(obj, names);
            rename_expr(idx, names);
        }
        Expr::Property { obj, .. } | Expr::NamedArgument { value: obj, .. } => rename_expr(obj, names),
        Expr::List(items) => {
            for item in items {
                rename_expr(item, names);
            }
        }
        Expr::Object(fields) => {
            for (_, value) in fields {
                rename_expr(value, names);
            }
        }
        Expr::New { class_name, args } => {
            if let Some(renamed) = names.get(class_name) {
                *class_name = renamed.clone();
            }
            for arg in args {
                rename_expr(arg, names);
            }
        }
        Expr::Literal(_) | Expr::Interpolated { .. } => {}
    }
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
