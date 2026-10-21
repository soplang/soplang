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
        if let Some(end_token) = preserved_import_end(&tokens, i) {
            let end = tokens
                .get(end_token)
                .map(|token| token.start)
                .unwrap_or(source.len());
            if start > cursor {
                out.push_str(&source[cursor..start]);
            }
            out.push_str(&source[start..end]);
            cursor = end;
            i = end_token;
            continue;
        }
        if start > cursor {
            out.push_str(&source[cursor..start]);
        }
        let import_end = tokens.get(i + 2).map(|t| t.start).unwrap_or(source.len());
        cursor = import_end;
        i += 2;

        let import_path = resolve_import_path(&base_dir, &path_tok.lexeme, source_path.is_some())?;
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

/// Expand file imports and retain one initializer for each namespaced module.
pub fn resolve_imports(
    stmts: Vec<Stmt>,
    source_path: Option<&Path>,
) -> Result<Vec<Stmt>, SoplangError> {
    ImportResolver::default().resolve(stmts, source_path)
}

#[derive(Clone)]
struct Module {
    loader: String,
    public: Vec<String>,
}

#[derive(Default, Clone)]
pub(crate) struct ImportResolver {
    flat_seen: HashSet<PathBuf>,
    active: HashSet<PathBuf>,
    modules: HashMap<PathBuf, Module>,
    caches: Vec<Stmt>,
    loaders: Vec<Stmt>,
    next_id: usize,
}

impl ImportResolver {
    pub(crate) fn resolve(
        &mut self,
        stmts: Vec<Stmt>,
        source_path: Option<&Path>,
    ) -> Result<Vec<Stmt>, SoplangError> {
        self.flat_seen.clear();
        self.active.clear();
        if let Some(path) = source_path.and_then(|path| path.canonicalize().ok()) {
            self.flat_seen.insert(path.clone());
            self.active.insert(path);
        }
        let body = self.flatten(stmts, source_path)?;
        self.active.clear();
        let mut result = std::mem::take(&mut self.caches);
        result.extend(std::mem::take(&mut self.loaders));
        result.extend(body);
        Ok(result)
    }

    fn flatten(
        &mut self,
        stmts: Vec<Stmt>,
        source_path: Option<&Path>,
    ) -> Result<Vec<Stmt>, SoplangError> {
        let base = source_path
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("."));
        let mut out = Vec::new();
        for stmt in stmts {
            match stmt {
                Stmt::Located { span, stmt } if matches!(stmt.unlocated(), Stmt::Import(_)) => {
                    out.extend(
                        self.flatten(vec![*stmt], source_path)
                            .map_err(|error| span.attach(error))?,
                    );
                }
                Stmt::Import(import) => {
                    let path = resolve_import_path(base, &import.path, source_path.is_some())?;
                    let canon = path
                        .canonicalize()
                        .map_err(|error| import_read_error(Path::new(&import.path), &error))?;
                    if import.alias.is_none() && import.selected.is_empty() {
                        if !self.flat_seen.insert(canon.clone()) {
                            continue;
                        }
                        let source = fs::read_to_string(&canon)
                            .map_err(|error| import_read_error(Path::new(&import.path), &error))?;
                        let parsed = super::parse_source(&source, Some(&path))?;
                        // Preserve flat imports' existing once-per-program behavior.
                        let inserted = self.active.insert(canon.clone());
                        out.extend(self.flatten(parsed, Some(&path))?);
                        if inserted {
                            self.active.remove(&canon);
                        }
                        continue;
                    }
                    let module = self.module(&canon, &path)?;
                    let value = Expr::Call {
                        name: module.loader,
                        args: Vec::new(),
                    };
                    if let Some(alias) = import.alias {
                        out.push(const_binding(alias, value));
                    } else {
                        for item in &import.selected {
                            if !module.public.contains(&item.name) {
                                return Err(import_error_ex(
                                    format!("'{}' lama soo saarin.", item.name),
                                    0,
                                    0,
                                    ErrorMeta::default().with_code(codes::E021_UNDECLARED),
                                ));
                            }
                        }
                        let hidden = format!("#module_import{}", self.next_id);
                        self.next_id += 1;
                        out.push(const_binding(hidden.clone(), value));
                        for item in import.selected {
                            out.push(const_binding(
                                item.alias.unwrap_or(item.name.clone()),
                                Expr::Property {
                                    obj: Box::new(Expr::Identifier(hidden.clone())),
                                    prop: item.name,
                                },
                            ));
                        }
                    }
                }
                stmt => out.push(stmt),
            }
        }
        Ok(out)
    }

    fn module(&mut self, path: &Path, source_path: &Path) -> Result<Module, SoplangError> {
        if self.active.contains(path) {
            return Err(import_error_ex(
                "Keen-ku waa wareeg; faylka ayaa dib ugu noqday fayl weli la soo dejinayo.",
                0,
                0,
                ErrorMeta::default().with_code(codes::E021_UNDECLARED),
            ));
        }
        if let Some(module) = self.modules.get(path) {
            return Ok(module.clone());
        }
        let id = self.next_id;
        self.next_id += 1;
        self.active.insert(path.to_path_buf());
        let source = fs::read_to_string(path).map_err(|error| import_read_error(path, &error))?;
        let parsed = super::parse_source(&source, Some(source_path))?;
        let mut body = self.flatten(parsed, Some(source_path))?;
        let public = export_names(&body)?;
        let classes: HashMap<String, String> = body
            .iter()
            .filter_map(|stmt| match stmt.unlocated() {
                Stmt::ClassDef { name, .. } => {
                    Some((name.clone(), format!("__soplang_module{id}_{name}")))
                }
                _ => None,
            })
            .collect();
        rename_stmts(&mut body, &classes);
        let fields = public
            .iter()
            .map(|name| (name.clone(), Expr::Identifier(name.clone())))
            .collect();
        body.push(Stmt::Return(Some(Expr::Object(fields))));
        let cache = format!("#module_cache{id}");
        let loader = format!("#module_load{id}");
        let loaded = format!("#module_loaded{id}");
        self.caches.push(Stmt::VarDecl {
            name: loaded.clone(),
            type_ann: TypeAnnotation::Bool,
            is_const: false,
            value: Expr::Literal(super::ast::Literal::Bool(false)),
            line: 0,
            col: 0,
        });
        self.caches.push(Stmt::VarDecl {
            name: cache.clone(),
            type_ann: TypeAnnotation::Walax,
            is_const: false,
            value: Expr::Object(Vec::new()),
            line: 0,
            col: 0,
        });
        self.loaders.push(Stmt::FuncDef {
            name: loader.clone(),
            params: Vec::new(),
            return_ann: TypeAnnotation::Walax,
            body: vec![
                Stmt::If {
                    cond: Expr::UnaryOp {
                        op: "!".into(),
                        expr: Box::new(Expr::Identifier(loaded.clone())),
                    },
                    then_body: vec![
                        Stmt::Assign {
                            target: Expr::Identifier(cache.clone()),
                            value: Expr::Invoke {
                                callee: Box::new(Expr::AnonymousFunc {
                                    params: Vec::new(),
                                    return_ann: TypeAnnotation::Walax,
                                    body,
                                }),
                                args: Vec::new(),
                            },
                            line: 0,
                            col: 0,
                        },
                        Stmt::Assign {
                            target: Expr::Identifier(loaded),
                            value: Expr::Literal(super::ast::Literal::Bool(true)),
                            line: 0,
                            col: 0,
                        },
                    ],
                    elseifs: Vec::new(),
                    else_body: None,
                },
                Stmt::Return(Some(Expr::Identifier(cache))),
            ],
        });
        self.active.remove(path);
        let module = Module { loader, public };
        self.modules.insert(path.to_path_buf(), module.clone());
        Ok(module)
    }
}

fn import_read_error(path: &Path, error: &std::io::Error) -> SoplangError {
    let error = file_read_error(path, error);
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
}

fn const_binding(name: String, value: Expr) -> Stmt {
    Stmt::VarDecl {
        name,
        type_ann: TypeAnnotation::Dynamic,
        is_const: true,
        value,
        line: 0,
        col: 0,
    }
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
            .filter(|name| !name.starts_with('_') && !name.starts_with('#'))
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

fn rename_stmts(stmts: &mut [Stmt], names: &HashMap<String, String>) {
    for stmt in stmts {
        rename_stmt(stmt, names);
    }
}

fn rename_stmt(stmt: &mut Stmt, names: &HashMap<String, String>) {
    match stmt {
        Stmt::Located { stmt, .. } => rename_stmt(stmt, names),
        Stmt::VarDecl {
            value, type_ann, ..
        } => {
            rename_type(type_ann, names);
            rename_expr(value, names);
        }
        Stmt::Assign { target, value, .. } => {
            rename_expr(target, names);
            rename_expr(value, names);
        }
        Stmt::FuncDef {
            params,
            return_ann,
            body,
            ..
        } => {
            rename_parameters(params, names);
            rename_type(return_ann, names);
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
        Stmt::If {
            cond,
            then_body,
            elseifs,
            else_body,
        } => {
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
        Stmt::Switch {
            expr,
            cases,
            default,
        } => {
            rename_expr(expr, names);
            for (value, body) in cases {
                rename_expr(value, names);
                rename_stmts(body, names);
            }
            if let Some(body) = default {
                rename_stmts(body, names);
            }
        }
        Stmt::For {
            start,
            end,
            step,
            body,
            ..
        } => {
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
        Stmt::TryCatch {
            try_body,
            catch_body,
            finally_body,
            ..
        } => {
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
        Expr::Identifier(_) => {}
        Expr::Call { args, .. } => {
            for arg in args {
                rename_expr(arg, names);
            }
        }
        Expr::BinaryOp { left, right, .. } => {
            rename_expr(left, names);
            rename_expr(right, names);
        }
        Expr::UnaryOp { expr, .. } => rename_expr(expr, names),
        Expr::Invoke { callee, args }
        | Expr::MethodCall {
            obj: callee, args, ..
        } => {
            rename_expr(callee, names);
            for arg in args {
                rename_expr(arg, names);
            }
        }
        Expr::AnonymousFunc {
            params,
            return_ann,
            body,
        } => {
            rename_parameters(params, names);
            rename_type(return_ann, names);
            rename_stmts(body, names);
        }
        Expr::Index { obj, idx } => {
            rename_expr(obj, names);
            rename_expr(idx, names);
        }
        Expr::Property { obj, .. } | Expr::NamedArgument { value: obj, .. } => {
            rename_expr(obj, names)
        }
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
        Expr::Interpolated { parts } => {
            for part in parts {
                if let super::ast::InterpolationPart::Value { expression, .. } = part {
                    rename_expr(expression, names);
                }
            }
        }
        Expr::Literal(_) => {}
    }
}

fn rename_parameters(params: &mut [super::ast::Param], names: &HashMap<String, String>) {
    for param in params {
        rename_type(&mut param.type_ann, names);
        if let Some(default) = &mut param.default {
            rename_expr(default, names);
        }
    }
}

fn rename_type(ty: &mut TypeAnnotation, names: &HashMap<String, String>) {
    match ty {
        TypeAnnotation::Class(name) => {
            if let Some(new) = names.get(name) {
                *name = new.clone();
            }
        }
        TypeAnnotation::SetOf(inner)
        | TypeAnnotation::ListOf(inner)
        | TypeAnnotation::ObjectOf(inner) => rename_type(inner, names),
        TypeAnnotation::TupleOf(types) | TypeAnnotation::Union(types) => {
            for ty in types {
                rename_type(ty, names);
            }
        }
        _ => {}
    }
}

fn preserved_import_end(tokens: &[crate::frontend::token::Token], index: usize) -> Option<usize> {
    let ident = |at: usize, word: Option<&str>| {
        tokens.get(at).is_some_and(|token| {
            token.kind == TokenType::Identifier && word.is_none_or(|word| token.lexeme == word)
        })
    };
    if ident(index + 2, Some("sida")) && ident(index + 3, None) {
        return Some(index + 4);
    }
    if !ident(index + 2, Some("ka")) {
        return None;
    }
    let mut cursor = index + 3;
    loop {
        if !ident(cursor, None) {
            return None;
        }
        cursor += 1;
        if ident(cursor, Some("sida")) && ident(cursor + 1, None) {
            cursor += 2;
        }
        if tokens
            .get(cursor)
            .is_some_and(|token| token.kind == TokenType::Comma)
        {
            cursor += 1;
            continue;
        }
        return Some(cursor);
    }
}

fn resolve_import_path(
    base_dir: &Path,
    path: &str,
    packages: bool,
) -> Result<PathBuf, SoplangError> {
    let candidate = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        base_dir.join(path)
    };
    if candidate.is_file() {
        return Ok(candidate);
    }
    if packages {
        if let Some(file) = crate::package::import_file(base_dir, path)? {
            return Ok(file);
        }
    }
    Ok(candidate)
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
