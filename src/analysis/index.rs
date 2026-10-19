//! Source identities for editor symbols. Runtime slots are deliberately unused.
use super::SourceFile;
use crate::frontend::{
    ast::{Expr, Literal, Param, Stmt, TypeAnnotation},
    Lexer, Token, TokenType as K,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Location {
    pub path: PathBuf,
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Debug)]
pub struct Symbol {
    pub id: String,
    pub name: String,
    pub location: Location,
    pub kind: u32,
    pub ty: String,
    pub scope: usize,
    pub parameters: Option<Vec<String>>,
    pub returns: String,
    pub owner: Option<String>,
    pub constant: bool,
    pub parameter: bool,
}
#[derive(Clone, Debug)]
pub struct Occurrence {
    pub location: Location,
    pub symbol: String,
    pub scope: usize,
    pub declaration: bool,
    pub write: bool,
}
#[derive(Clone, Debug)]
pub struct Scope {
    pub path: PathBuf,
    pub start: usize,
    pub end: usize,
    pub parent: Option<usize>,
    pub class: Option<String>,
}
#[derive(Default, Debug, Clone)]
pub struct Index {
    pub symbols: Vec<Symbol>,
    pub occurrences: Vec<Occurrence>,
    pub scopes: Vec<Scope>,
    pub parents: HashMap<String, String>,
    pub tokens: HashMap<PathBuf, Vec<Token>>,
    pub unresolved_members: HashSet<String>,
    by_id: HashMap<String, usize>,
    by_name: HashMap<(usize, String), Vec<usize>>,
}
pub fn type_name(ty: &TypeAnnotation) -> String {
    match ty {
        TypeAnnotation::Abn => "abn",
        TypeAnnotation::Jajab => "jajab",
        TypeAnnotation::Qoraal => "qoraal",
        TypeAnnotation::Bool => "bool",
        TypeAnnotation::Teed => "teed",
        TypeAnnotation::Walax => "walax",
        TypeAnnotation::Dynamic => "door",
        TypeAnnotation::Class(name) => name,
    }
    .to_owned()
}
fn inferred(expr: &Expr) -> String {
    match expr.unlocated() {
        Expr::Literal(l) => match l {
            Literal::Int(_) => "abn",
            Literal::Float(_) => "jajab",
            Literal::Str(_) => "qoraal",
            Literal::Bool(_) => "bool",
            Literal::Null => "door",
        }
        .into(),
        Expr::List(_) => "teed".into(),
        Expr::Object(_) => "walax".into(),
        Expr::New { class_name, .. } => class_name.clone(),
        Expr::AnonymousFunc { .. } => "hawl".into(),
        _ => "door".into(),
    }
}
fn offset(source: &str, line: usize, col: usize) -> usize {
    let start = source
        .split_inclusive('\n')
        .take(line.saturating_sub(1))
        .map(str::len)
        .sum::<usize>();
    start
        + source
            .get(start..)
            .unwrap_or("")
            .chars()
            .take(col.saturating_sub(1))
            .map(char::len_utf8)
            .sum::<usize>()
}
impl Index {
    pub fn build(files: &[SourceFile]) -> Self {
        let mut index = Self::default();
        index.scopes.push(Scope {
            path: PathBuf::new(),
            start: 0,
            end: usize::MAX,
            parent: None,
            class: None,
        });
        for file in files {
            if let Ok(tokens) = Lexer::new(&file.text).tokenize() {
                index.tokens.insert(file.path.clone(), tokens);
            }
        }
        for file in files {
            index.statements(&file.statements, file, 0, 0, None);
        }
        let mut assigned = HashSet::new();
        for file in files {
            if let Some(tokens) = index.tokens.get(&file.path) {
                for pair in tokens.windows(2) {
                    if pair[1].kind == K::Assign {
                        let scope = index.scope_at(&file.path, pair[0].start);
                        if let Some(symbol) =
                            index.resolve(&pair[0].lexeme, scope, pair[0].start, &file.path)
                        {
                            if symbol.location.start != pair[0].start {
                                assigned.insert(symbol.id.clone());
                            }
                        }
                    }
                }
            }
        }
        for symbol in &mut index.symbols {
            if assigned.contains(&symbol.id) && symbol.returns == "inferred" {
                symbol.ty = "door".into();
            }
        }
        let declarations: HashSet<Location> =
            index.symbols.iter().map(|s| s.location.clone()).collect();
        for file in files {
            let tokens = index.tokens.get(&file.path).cloned().unwrap_or_default();
            if tokens.is_empty() {
                continue;
            }
            let mut parser = crate::frontend::Parser::new(tokens.clone())
                .with_source(&file.text, Some(&file.path));
            parser.parse_partial();
            let contextual = parser.contextual_keyword_offsets();
            for (i, token) in tokens.iter().enumerate() {
                if contextual.contains(&token.start) {
                    continue;
                }
                if !matches!(token.kind, K::Identifier | K::Nafta | K::Waalid)
                    && !declarations.contains(&Location {
                        path: file.path.clone(),
                        start: token.start,
                        end: token.start + token.lexeme.len(),
                    })
                {
                    continue;
                }
                let location = Location {
                    path: file.path.clone(),
                    start: token.start,
                    end: token.start + token.lexeme.len(),
                };
                if declarations.contains(&location) {
                    continue;
                }
                let scope = index.scope_at(&file.path, token.start);
                if tokens.get(i + 1).is_some_and(|t| t.kind == K::Colon)
                    && tokens
                        .get(i.wrapping_sub(1))
                        .is_some_and(|t| matches!(t.kind, K::LBrace | K::Comma))
                {
                    continue;
                }
                let id = if tokens
                    .get(i.wrapping_sub(1))
                    .is_some_and(|t| t.kind == K::Dot)
                {
                    let recv = tokens.get(i.wrapping_sub(2));
                    let ty = recv
                        .map(|recv| index.receiver_type(&file.path, recv.start, scope))
                        .unwrap_or_default();
                    let found = index.method(&ty, &token.lexeme).map(|s| s.id.clone());
                    if found.is_none() {
                        index.unresolved_members.insert(token.lexeme.clone());
                    }
                    found
                } else {
                    index
                        .resolve(&token.lexeme, scope, token.start, &file.path)
                        .map(|s| s.id.clone())
                };
                if let Some(symbol) = id {
                    index.occurrences.push(Occurrence {
                        location,
                        symbol,
                        scope,
                        declaration: false,
                        write: tokens.get(i + 1).is_some_and(|t| t.kind == K::Assign),
                    });
                }
            }
        }
        // Inferred receiver types become uncertain after writes. Do not offer a confident member identity.
        let assigned: HashSet<String> = index
            .occurrences
            .iter()
            .filter(|o| o.write && !o.declaration)
            .map(|o| o.symbol.clone())
            .collect();
        for symbol in &mut index.symbols {
            if assigned.contains(&symbol.id) && !symbol.constant && symbol.returns == "inferred" {
                symbol.ty = "door".into();
            }
        }
        index
    }
    /// Simulate the binding change for every affected lexical occurrence, including
    /// reverse captures in descendants. A rename must preserve every identity.
    pub fn rename_preserves_bindings(&self, id: &str, new: &str) -> bool {
        let Some(target) = self.symbol(id) else {
            return true;
        };
        if self
            .symbols
            .iter()
            .any(|s| s.id != id && s.scope == target.scope && s.owner.is_none() && s.name == new)
        {
            return false;
        }
        let mut renamed = self.clone();
        let index = renamed.by_id[id];
        if let Some(entries) = renamed
            .by_name
            .get_mut(&(target.scope, target.name.clone()))
        {
            entries.retain(|i| *i != index);
        }
        renamed
            .by_name
            .entry((target.scope, new.to_owned()))
            .or_default()
            .push(index);
        renamed.symbols[index].name = new.to_owned();
        for occurrence in &self.occurrences {
            let Some(symbol) = self.symbol(&occurrence.symbol) else {
                continue;
            };
            if symbol.owner.is_some()
                || occurrence.declaration
                || symbol.id != id && symbol.name != new
            {
                continue;
            }
            let name = if symbol.id == id { new } else { &symbol.name };
            if renamed
                .resolve(
                    name,
                    occurrence.scope,
                    occurrence.location.start,
                    &occurrence.location.path,
                )
                .map(|s| s.id.as_str())
                != Some(symbol.id.as_str())
            {
                return false;
            }
        }
        true
    }
    pub fn symbol(&self, id: &str) -> Option<&Symbol> {
        self.by_id.get(id).and_then(|i| self.symbols.get(*i))
    }
    pub fn at(&self, path: &Path, offset: usize) -> Option<&Occurrence> {
        self.occurrences.iter().find(|o| {
            o.location.path == path && o.location.start <= offset && offset < o.location.end
        })
    }
    pub fn scope_at(&self, path: &Path, offset: usize) -> usize {
        self.scopes
            .iter()
            .enumerate()
            .filter(|(_, s)| s.path == path && s.start <= offset && offset <= s.end)
            .min_by_key(|(_, s)| s.end - s.start)
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
    pub fn resolve(
        &self,
        name: &str,
        mut scope: usize,
        offset: usize,
        path: &Path,
    ) -> Option<&Symbol> {
        let in_function = scope != 0;
        loop {
            if let Some(s) = self
                .by_name
                .get(&(scope, name.to_owned()))
                .and_then(|entries| {
                    entries
                        .iter()
                        .rev()
                        .filter_map(|i| self.symbols.get(*i))
                        .find(|s| {
                            s.owner.is_none()
                                && (s.location.path != path
                                    || s.location.start <= offset
                                    || s.parameters.is_some()
                                    || s.kind == 5
                                    || in_function && s.scope == 0)
                        })
                })
            {
                return Some(s);
            }
            scope = self.scopes.get(scope)?.parent?;
        }
    }
    pub fn visible(&self, path: &Path, offset: usize) -> Vec<&Symbol> {
        let mut found = HashSet::new();
        let mut scope = self.scope_at(path, offset);
        let mut result = vec![];
        loop {
            for s in self.symbols.iter().rev().filter(|s| {
                s.scope == scope
                    && s.owner.is_none()
                    && (s.location.path != path
                        || s.location.start <= offset
                        || s.parameters.is_some()
                        || s.kind == 5)
            }) {
                if found.insert(s.name.clone()) {
                    result.push(s);
                }
            }
            match self.scopes[scope].parent {
                Some(parent) => scope = parent,
                None => break,
            }
        }
        result
    }
    pub fn method(&self, ty: &str, name: &str) -> Option<&Symbol> {
        let mut ty = ty;
        let mut seen = HashSet::new();
        loop {
            if !seen.insert(ty) {
                return None;
            }
            if let Some(s) = self
                .symbols
                .iter()
                .find(|s| s.owner.as_deref() == Some(ty) && s.name == name)
                .or_else(|| {
                    let other = match name {
                        "dhis" => "dhaw",
                        "dhaw" => "dhis",
                        _ => return None,
                    };
                    self.symbols
                        .iter()
                        .find(|s| s.owner.as_deref() == Some(ty) && s.name == other)
                })
            {
                return Some(s);
            }
            ty = self.parents.get(ty)?;
        }
    }
    pub fn receiver_type(&self, path: &Path, start: usize, scope: usize) -> String {
        let Some(t) = self
            .tokens
            .get(path)
            .and_then(|tokens| tokens.iter().find(|t| t.start == start))
        else {
            return "door".into();
        };
        if t.lexeme == "nafta" || t.lexeme == "waalid" {
            let mut cursor = Some(scope);
            while let Some(i) = cursor {
                if let Some(class) = &self.scopes[i].class {
                    return if t.lexeme == "waalid" {
                        self.parents.get(class).cloned().unwrap_or_default()
                    } else {
                        class.clone()
                    };
                }
                cursor = self.scopes[i].parent;
            }
            return "door".into();
        }
        match t.kind {
            K::String => "qoraal".into(),
            K::Number => "abn".into(),
            _ => self
                .resolve(&t.lexeme, scope, start, path)
                .map(|s| s.ty.clone())
                .unwrap_or_else(|| "door".into()),
        }
    }
    fn anchor(&self, file: &SourceFile, start: usize, name: &str) -> Option<Token> {
        {
            let tokens = self.tokens.get(&file.path)?;
            tokens[tokens.partition_point(|t| t.start < start)..]
                .iter()
                .find(|t| t.lexeme == name)
                .cloned()
        }
    }
    fn add(
        &mut self,
        file: &SourceFile,
        token: &Token,
        name: &str,
        scope: usize,
        kind: u32,
        ty: String,
        params: Option<Vec<String>>,
        returns: String,
        owner: Option<String>,
        constant: bool,
    ) -> String {
        let location = Location {
            path: file.path.clone(),
            start: token.start,
            end: token.start + token.lexeme.len(),
        };
        let id = format!("{}#{}", file.path.display(), token.start);
        self.occurrences.push(Occurrence {
            location: location.clone(),
            symbol: id.clone(),
            scope,
            declaration: true,
            write: true,
        });
        self.by_id.insert(id.clone(), self.symbols.len());
        self.by_name
            .entry((scope, name.to_owned()))
            .or_default()
            .push(self.symbols.len());
        self.symbols.push(Symbol {
            id: id.clone(),
            name: name.into(),
            location,
            kind,
            ty,
            scope,
            parameters: params,
            returns,
            owner,
            constant,
            parameter: false,
        });
        id
    }
    fn body_scope(
        &mut self,
        file: &SourceFile,
        start: usize,
        parent: usize,
        class: Option<String>,
    ) -> usize {
        let tokens = self.tokens.get(&file.path).cloned().unwrap_or_default();
        let open = tokens
            .iter()
            .position(|t| t.start >= start && t.kind == K::LBrace);
        let (begin, end) = if let Some(open) = open {
            let mut depth = 0;
            let mut end = file.text.len();
            for token in &tokens[open..] {
                match token.kind {
                    K::LBrace => depth += 1,
                    K::RBrace => {
                        depth -= 1;
                        if depth == 0 {
                            end = token.start;
                            break;
                        }
                    }
                    _ => (),
                }
            }
            (start, end)
        } else {
            (start, file.text.len())
        };
        let id = self.scopes.len();
        self.scopes.push(Scope {
            path: file.path.clone(),
            start: begin,
            end,
            parent: Some(parent),
            class,
        });
        id
    }
    fn function(
        &mut self,
        file: &SourceFile,
        start: usize,
        scope: usize,
        params: &[Param],
        ret: &TypeAnnotation,
        body: &[Stmt],
        class: Option<&str>,
    ) {
        let inner = self.body_scope(file, start, scope, class.map(str::to_owned));
        let mut cursor = self
            .tokens
            .get(&file.path)
            .and_then(|t| t.iter().find(|t| t.start >= start && t.kind == K::LParen))
            .map(|t| t.start + 1)
            .unwrap_or(start);
        for param in params {
            let token = self.tokens.get(&file.path).and_then(|tokens| {
                tokens[tokens.partition_point(|t| t.start < cursor)..]
                    .windows(2)
                    .find(|pair| {
                        pair[0].lexeme == param.name && matches!(pair[1].kind, K::Comma | K::RParen)
                    })
                    .map(|pair| pair[0].clone())
            });
            if let Some(token) = token {
                self.add(
                    file,
                    &token,
                    &param.name,
                    inner,
                    13,
                    type_name(&param.type_ann),
                    None,
                    String::new(),
                    None,
                    false,
                );
                if let Some(symbol) = self.symbols.last_mut() {
                    symbol.parameter = true;
                }
                cursor = token.start + token.lexeme.len();
            }
        }
        let _ = ret;
        self.statements(body, file, inner, start, class);
    }
    fn statements(
        &mut self,
        stmts: &[Stmt],
        file: &SourceFile,
        scope: usize,
        mut start: usize,
        class: Option<&str>,
    ) {
        for stmt in stmts {
            if let Stmt::Located { span, .. } = stmt {
                start = offset(&file.text, span.line, span.col);
            }
            match stmt.unlocated() {
                Stmt::VarDecl {
                    name,
                    type_ann,
                    is_const,
                    value,
                    ..
                } => {
                    let declaration = self.tokens.get(&file.path).and_then(|tokens| {
                        tokens[tokens.partition_point(|t| t.start < start)..]
                            .windows(2)
                            .find(|pair| pair[0].lexeme == *name && pair[1].kind == K::Assign)
                            .map(|pair| pair[0].clone())
                    });
                    if let Some(token) = declaration {
                        let inferred_type = *type_ann == TypeAnnotation::Dynamic;
                        let ty = if inferred_type {
                            inferred(value)
                        } else {
                            type_name(type_ann)
                        };
                        let (params, ret) = if let Expr::AnonymousFunc {
                            params, return_ann, ..
                        } = value.unlocated()
                        {
                            (
                                Some(
                                    params
                                        .iter()
                                        .map(|p| format!("{} {}", type_name(&p.type_ann), p.name))
                                        .collect(),
                                ),
                                type_name(return_ann),
                            )
                        } else {
                            (
                                None,
                                if inferred_type {
                                    "inferred".into()
                                } else {
                                    String::new()
                                },
                            )
                        };
                        self.add(
                            file,
                            &token,
                            name,
                            scope,
                            if *is_const { 14 } else { 13 },
                            ty,
                            params,
                            ret,
                            None,
                            *is_const,
                        );
                    }
                    self.expression(value, file, scope, start, class);
                }
                Stmt::FuncDef {
                    name,
                    params,
                    return_ann,
                    body,
                } => {
                    if let Some(token) = self.anchor(file, start, name) {
                        self.add(
                            file,
                            &token,
                            name,
                            scope,
                            if class.is_some() && scope == 0 { 6 } else { 12 },
                            "hawl".into(),
                            Some(
                                params
                                    .iter()
                                    .map(|p| format!("{} {}", type_name(&p.type_ann), p.name))
                                    .collect(),
                            ),
                            type_name(return_ann),
                            if scope == 0 {
                                class.map(str::to_owned)
                            } else {
                                None
                            },
                            true,
                        );
                    }
                    self.function(file, start, scope, params, return_ann, body, class);
                }
                Stmt::ClassDef { name, parent, body } => {
                    if let Some(token) = self.anchor(file, start, name) {
                        self.add(
                            file,
                            &token,
                            name,
                            scope,
                            5,
                            name.clone(),
                            None,
                            String::new(),
                            None,
                            true,
                        );
                    }
                    if let Some(parent) = parent {
                        self.parents.insert(name.clone(), parent.clone());
                    }
                    self.statements(body, file, scope, start, Some(name));
                }
                Stmt::For {
                    var,
                    start: from,
                    end,
                    step,
                    body,
                } => {
                    if self.resolve(var, scope, start, &file.path).is_none() {
                        if let Some(token) = self.anchor(file, start, var) {
                            self.add(
                                file,
                                &token,
                                var,
                                scope,
                                13,
                                "abn".into(),
                                None,
                                String::new(),
                                None,
                                false,
                            );
                        }
                    }
                    self.expression(from, file, scope, start, class);
                    self.expression(end, file, scope, start, class);
                    if let Some(step) = step {
                        self.expression(step, file, scope, start, class);
                    }
                    self.statements(body, file, scope, start, class);
                }
                Stmt::TryCatch {
                    try_body,
                    err_var,
                    err_details,
                    catch_body,
                    finally_body,
                } => {
                    self.statements(try_body, file, scope, start, class);
                    let tokens = self.tokens.get(&file.path).cloned().unwrap_or_default();
                    let mut depth = 0usize;
                    let mut catch = start;
                    for token in tokens.iter().filter(|t| t.start >= start) {
                        match token.kind {
                            K::LBrace => depth += 1,
                            K::RBrace => depth = depth.saturating_sub(1),
                            K::Qabo if depth == 0 => {
                                catch = token.start;
                                break;
                            }
                            _ => (),
                        }
                    }
                    let inner = self.body_scope(file, catch, scope, class.map(str::to_owned));
                    for name in err_var.iter().chain(err_details.iter()) {
                        if let Some(token) = self.anchor(file, catch, name) {
                            self.add(
                                file,
                                &token,
                                name,
                                inner,
                                13,
                                "door".into(),
                                None,
                                String::new(),
                                None,
                                false,
                            );
                        }
                    }
                    self.statements(catch_body, file, inner, catch, class);
                    self.statements(finally_body, file, scope, start, class);
                }
                Stmt::If {
                    cond,
                    then_body,
                    elseifs,
                    else_body,
                } => {
                    self.expression(cond, file, scope, start, class);
                    self.statements(then_body, file, scope, start, class);
                    for (cond, body) in elseifs {
                        self.expression(cond, file, scope, start, class);
                        self.statements(body, file, scope, start, class);
                    }
                    if let Some(body) = else_body {
                        self.statements(body, file, scope, start, class);
                    }
                }
                Stmt::Switch {
                    expr,
                    cases,
                    default,
                } => {
                    self.expression(expr, file, scope, start, class);
                    for (expr, body) in cases {
                        self.expression(expr, file, scope, start, class);
                        self.statements(body, file, scope, start, class);
                    }
                    if let Some(body) = default {
                        self.statements(body, file, scope, start, class);
                    }
                }
                Stmt::While { cond, body } => {
                    self.expression(cond, file, scope, start, class);
                    self.statements(body, file, scope, start, class);
                }
                Stmt::Block(body) => self.statements(body, file, scope, start, class),
                Stmt::Expr(expr) | Stmt::Return(Some(expr)) | Stmt::Throw(expr) => {
                    self.expression(expr, file, scope, start, class)
                }
                Stmt::Assign { target, value, .. } => {
                    self.expression(target, file, scope, start, class);
                    self.expression(value, file, scope, start, class);
                }
                _ => (),
            }
        }
    }
    fn expression(
        &mut self,
        expr: &Expr,
        file: &SourceFile,
        scope: usize,
        mut start: usize,
        class: Option<&str>,
    ) {
        if let Expr::Located { span, .. } = expr {
            start = offset(&file.text, span.line, span.col);
        }
        match expr.unlocated() {
            Expr::AnonymousFunc {
                params,
                return_ann,
                body,
            } => self.function(file, start, scope, params, return_ann, body, class),
            Expr::BinaryOp { left, right, .. } => {
                self.expression(left, file, scope, start, class);
                self.expression(right, file, scope, start, class);
            }
            Expr::UnaryOp { expr, .. } => self.expression(expr, file, scope, start, class),
            Expr::Call { args, .. } | Expr::New { args, .. } => {
                for arg in args {
                    self.expression(arg, file, scope, start, class)
                }
            }
            Expr::MethodCall { obj, args, .. } | Expr::Invoke { callee: obj, args } => {
                self.expression(obj, file, scope, start, class);
                for arg in args {
                    self.expression(arg, file, scope, start, class)
                }
            }
            Expr::Index { obj, idx } => {
                self.expression(obj, file, scope, start, class);
                self.expression(idx, file, scope, start, class)
            }
            Expr::Property { obj, .. } => self.expression(obj, file, scope, start, class),
            Expr::List(items) => {
                for item in items {
                    self.expression(item, file, scope, start, class)
                }
            }
            Expr::Object(items) => {
                for (_, item) in items {
                    self.expression(item, file, scope, start, class)
                }
            }
            _ => (),
        }
    }
}
