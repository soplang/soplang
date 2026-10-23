//! Source identities for editor symbols. Runtime slots are deliberately unused.
mod modules;
mod native;
mod types;

use super::SourceFile;
use crate::frontend::{
    ast::{Expr, InterpolationPart, Literal, Param, Stmt, TypeAnnotation},
    Token, TokenType as K,
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
    file_scopes: HashMap<PathBuf, usize>,
    class_scopes: HashSet<usize>,
    class_keys: HashMap<String, String>,
    function_scopes: HashSet<usize>,
    exports: HashMap<PathBuf, HashMap<String, usize>>,
    module_aliases: HashMap<String, PathBuf>,
    alias_targets: HashMap<String, String>,
    member_locations: HashSet<Location>,
    value_hints: HashMap<String, Vec<(Expr, usize)>>,
    native: HashMap<String, &'static crate::native_api::Api>,
    value_keywords: HashSet<(PathBuf, usize)>,
    reassigned: HashSet<String>,
    by_id: HashMap<String, usize>,
    by_name: HashMap<(usize, String), Vec<usize>>,
}
pub fn type_name(ty: &TypeAnnotation) -> String {
    ty.to_string()
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
        Expr::Interpolated { .. } => "qoraal".into(),
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
fn index_tokens(tokens: Vec<Token>) -> Vec<Token> {
    fn visit(tokens: Vec<Token>, out: &mut Vec<Token>) {
        for token in tokens {
            let parts = token.format_parts.clone();
            out.push(token);
            for part in parts {
                if let crate::frontend::token::FormatPart::Field { tokens, .. } = part {
                    visit(
                        tokens
                            .into_iter()
                            .filter(|token| token.kind != K::Eof)
                            .collect(),
                        out,
                    );
                }
            }
        }
    }
    let mut out = Vec::new();
    visit(tokens, &mut out);
    out.sort_by_key(|token| token.start);
    out
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
        index.install_native();
        let mut contextual_by_file = HashMap::new();
        for file in files {
            let (_, tokens, contextual) = super::editor_syntax(&file.text, &file.path);
            index.tokens.insert(file.path.clone(), index_tokens(tokens));
            contextual_by_file.insert(file.path.clone(), contextual);
        }
        for file in files {
            let scope = index.scopes.len();
            index.scopes.push(Scope {
                path: file.path.clone(),
                start: 0,
                end: file.text.len(),
                parent: Some(0),
                class: None,
            });
            index.file_scopes.insert(file.path.clone(), scope);
            index.statements(&file.statements, file, scope, 0, None);
        }
        index.bind_modules(files);
        let mut assigned = HashSet::new();
        for file in files {
            if let Some(tokens) = index.tokens.get(&file.path) {
                for (position, pair) in tokens.windows(2).enumerate() {
                    if pair[1].kind == K::Assign
                        && !tokens
                            .get(position.wrapping_sub(1))
                            .is_some_and(|token| token.kind == K::Dot)
                    {
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
        index.reassigned = assigned.clone();
        for symbol in &mut index.symbols {
            if assigned.contains(&symbol.id)
                && symbol.owner.is_none()
                && symbol.returns == "inferred"
            {
                symbol.ty = "door".into();
                index.value_hints.remove(&symbol.id);
            }
        }
        index.normalize_types();
        let declarations: HashSet<Location> = index
            .occurrences
            .iter()
            .map(|occurrence| occurrence.location.clone())
            .collect();
        for file in files {
            let tokens = index.tokens.get(&file.path).cloned().unwrap_or_default();
            if tokens.is_empty() {
                continue;
            }
            let contextual = &contextual_by_file[&file.path];
            for (i, token) in tokens.iter().enumerate() {
                if contextual.contains(&token.start) {
                    continue;
                }
                let member = i > 0 && tokens[i - 1].kind == K::Dot;
                let value_keyword = index
                    .value_keywords
                    .contains(&(file.path.clone(), token.start));
                if !matches!(token.kind, K::Identifier | K::Nafta | K::Waalid)
                    && !member
                    && !value_keyword
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
                    if found.is_some() {
                        index.member_locations.insert(location.clone());
                    }
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
        index.reassigned = assigned.clone();
        for symbol in &mut index.symbols {
            if assigned.contains(&symbol.id)
                && symbol.owner.is_none()
                && !symbol.constant
                && symbol.returns == "inferred"
            {
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
        if target.owner.is_some()
            && (matches!(target.name.as_str(), "dhis" | "dhaw")
                || self.unresolved_members.contains(&target.name))
        {
            return false;
        }
        if let Some(owner) = &target.owner {
            let related = |left: &str, right: &str| {
                let ancestors = |start: &str, goal: &str| {
                    let mut next = Some(start);
                    let mut seen = HashSet::new();
                    while let Some(current) = next {
                        if current == goal {
                            return true;
                        }
                        if !seen.insert(current) {
                            break;
                        }
                        next = self.parents.get(current).map(String::as_str);
                    }
                    false
                };
                ancestors(left, right) || ancestors(right, left)
            };
            if self.symbols.iter().any(|symbol| {
                symbol.id != id
                    && symbol
                        .owner
                        .as_deref()
                        .is_some_and(|other| related(owner, other))
                    && (symbol.name == new || symbol.name == target.name)
            }) {
                return false;
            }
        }
        if self.symbols.iter().any(|symbol| {
            symbol.id != id
                && symbol.owner == target.owner
                && symbol.name == new
                && (target.owner.is_some() || symbol.scope == target.scope)
        }) {
            return false;
        }
        let mut renamed = self.clone();
        let index = renamed.by_id[id];
        let bindings: Vec<_> = renamed
            .by_name
            .iter()
            .filter(|((_, name), entries)| name == &target.name && entries.contains(&index))
            .map(|(key, _)| key.clone())
            .collect();
        for (scope, old) in bindings {
            if renamed
                .by_name
                .get(&(scope, new.into()))
                .is_some_and(|entries| entries.iter().any(|entry| *entry != index))
            {
                return false;
            }
            renamed
                .by_name
                .get_mut(&(scope, old))
                .unwrap()
                .retain(|entry| *entry != index);
            renamed
                .by_name
                .entry((scope, new.into()))
                .or_default()
                .push(index);
        }
        for exports in renamed.exports.values_mut() {
            if exports.get(&target.name) == Some(&index) {
                if exports.get(new).is_some_and(|entry| *entry != index) {
                    return false;
                }
                exports.remove(&target.name);
                exports.insert(new.into(), index);
            }
        }
        renamed.symbols[index].name = new.to_owned();
        for occurrence in &self.occurrences {
            let Some(symbol) = self.symbol(&occurrence.symbol) else {
                continue;
            };
            if symbol.owner.is_some()
                || self.member_locations.contains(&occurrence.location)
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
            .min_by_key(|(index, s)| (s.end - s.start, std::cmp::Reverse(*index)))
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
        let mut cursor = Some(scope);
        let mut in_function = false;
        while let Some(current) = cursor {
            in_function |= self.function_scopes.contains(&current);
            cursor = self.scopes.get(current).and_then(|scope| scope.parent);
        }
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
                                    || in_function
                                        && self.file_scopes.values().any(|root| *root == s.scope))
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
            for s in self
                .by_name
                .iter()
                .filter(|((owner, _), _)| *owner == scope)
                .flat_map(|(_, entries)| entries.iter().rev())
                .filter_map(|index| self.symbols.get(*index))
                .filter(|s| {
                    s.owner.is_none()
                        && (s.location.path != path
                            || s.location.start <= offset
                            || s.parameters.is_some()
                            || s.kind == 5)
                })
            {
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
        let ty = ty.strip_suffix('?').unwrap_or(ty);
        if let Some(owner) = crate::native_api::base(ty) {
            return self
                .symbols
                .iter()
                .find(|symbol| symbol.owner.as_deref() == Some(owner) && symbol.name == name);
        }
        if let Some(module) = self.module_aliases.get(ty) {
            return self
                .exports
                .get(module)?
                .get(name)
                .and_then(|index| self.symbols.get(*index));
        }
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
        // Defaults may contain object literals and nested functions. Locate the
        // body after the complete parameter list instead of its first brace.
        let tokens = self.tokens.get(&file.path).cloned().unwrap_or_default();
        let open = tokens
            .iter()
            .position(|token| token.start >= start && token.kind == K::Hawl)
            .and_then(|keyword| {
                tokens
                    .iter()
                    .enumerate()
                    .skip(keyword + 1)
                    .find(|(_, token)| token.kind == K::LParen)
                    .map(|(index, _)| index)
            });
        let mut segments = Vec::new();
        let mut body_start = start;
        if let Some(open) = open {
            let mut depth = 0usize;
            let mut begin = open + 1;
            for (index, token) in tokens.iter().enumerate().skip(begin) {
                match token.kind {
                    K::RParen if depth == 0 => {
                        segments.push(begin..index);
                        body_start = tokens[index + 1..]
                            .iter()
                            .find(|token| token.kind == K::LBrace)
                            .map(|token| token.start)
                            .unwrap_or(start);
                        break;
                    }
                    K::Comma if depth == 0 => {
                        segments.push(begin..index);
                        begin = index + 1;
                    }
                    K::LParen | K::LBracket | K::LBrace => depth += 1,
                    K::RParen | K::RBracket | K::RBrace => depth = depth.saturating_sub(1),
                    _ => (),
                }
            }
        }
        let inner = self.body_scope(file, body_start, scope, class.map(str::to_owned));
        self.scopes[inner].start = start;
        self.function_scopes.insert(inner);
        for (param, segment) in params.iter().zip(segments) {
            let token = tokens[segment]
                .iter()
                .take_while(|token| token.kind != K::Assign)
                .filter(|token| token.lexeme == param.name)
                .last()
                .cloned();
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
            }
        }
        for param in params {
            if let Some(default) = &param.default {
                self.expression(default, file, inner, start, class);
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
                                Some(params.iter().map(parameter_label).collect()),
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
                        let id = self.add(
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
                        if inferred_type {
                            self.value_hints.insert(id, vec![(value.clone(), scope)]);
                        }
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
                            if self.class_scopes.contains(&scope) {
                                6
                            } else {
                                12
                            },
                            "hawl".into(),
                            Some(params.iter().map(parameter_label).collect()),
                            type_name(return_ann),
                            if self.class_scopes.contains(&scope) {
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
                        let id = self.add(
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
                        let key = format!("@class{}", self.by_id[&id]);
                        self.class_keys.insert(key.clone(), id);
                        self.symbols.last_mut().unwrap().ty = key.clone();
                        if let Some(parent) = parent {
                            self.parents.insert(key.clone(), parent.clone());
                        }
                        let inner = self.body_scope(file, start, scope, Some(key.clone()));
                        self.class_scopes.insert(inner);
                        self.statements(body, file, inner, start, Some(&key));
                    }
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
                Stmt::ForEach {
                    vars,
                    iterable,
                    body,
                } => {
                    self.expression(iterable, file, scope, start, class);
                    for name in vars {
                        if self.resolve(name, scope, start, &file.path).is_none() {
                            if let Some(token) = self.anchor(file, start, name) {
                                self.add(
                                    file,
                                    &token,
                                    name,
                                    scope,
                                    13,
                                    "door".into(),
                                    None,
                                    String::new(),
                                    None,
                                    false,
                                );
                            }
                        }
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
                Stmt::Block(body) | Stmt::Test { body, .. } | Stmt::Fixture { body, .. } => {
                    self.statements(body, file, scope, start, class)
                }
                Stmt::Expr(expr) | Stmt::Return(Some(expr)) | Stmt::Throw(expr) => {
                    self.expression(expr, file, scope, start, class)
                }
                Stmt::Assign { target, value, .. } => {
                    if let (Some(owner), Expr::Property { obj, prop: name }) =
                        (class, target.unlocated())
                    {
                        if matches!(obj.unlocated(), Expr::Identifier(receiver) if receiver == "nafta")
                        {
                            let existing = self
                                .symbols
                                .iter()
                                .find(|symbol| {
                                    symbol.owner.as_deref() == Some(owner) && symbol.name == *name
                                })
                                .map(|symbol| symbol.id.clone());
                            let id = existing.or_else(|| {
                                self.anchor(file, start, name).map(|token| {
                                    self.add(
                                        file,
                                        &token,
                                        name,
                                        scope,
                                        8,
                                        "door".into(),
                                        None,
                                        "inferred".into(),
                                        Some(owner.into()),
                                        false,
                                    )
                                })
                            });
                            if let Some(id) = id {
                                self.value_hints
                                    .entry(id)
                                    .or_default()
                                    .push((value.clone(), scope));
                            }
                        }
                    }
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
        if let Expr::Identifier(name) | Expr::Call { name, .. } = expr.unlocated() {
            if let Some(token) = self.anchor(file, start, name) {
                self.value_keywords.insert((file.path.clone(), token.start));
            }
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
            Expr::Interpolated { parts } => {
                for part in parts {
                    if let InterpolationPart::Value { expression, .. } = part {
                        self.expression(expression, file, scope, start, class);
                    }
                }
            }
            Expr::NamedArgument { value: expr, .. } | Expr::UnaryOp { expr, .. } => {
                self.expression(expr, file, scope, start, class)
            }
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

fn parameter_label(param: &Param) -> String {
    let default = param
        .default
        .as_ref()
        .map(|value| format!(" = {}", value))
        .unwrap_or_default();
    format!(
        "{} {}{}{}",
        type_name(&param.type_ann),
        if param.keyword_rest {
            "**"
        } else if param.variadic {
            "*"
        } else {
            ""
        },
        param.name,
        default
    )
}
