//! Import bindings use source identities; compiled runtime slots are irrelevant.

use super::*;

impl Index {
    pub(super) fn bind_modules(&mut self, files: &[SourceFile]) {
        let by_path: HashMap<_, _> = files.iter().map(|file| (file.path.clone(), file)).collect();
        let mut active = HashSet::new();
        let mut complete = HashSet::new();
        for file in files {
            self.bind_file(file, &by_path, &mut active, &mut complete);
        }
    }

    fn bind_file(
        &mut self,
        file: &SourceFile,
        files: &HashMap<PathBuf, &SourceFile>,
        active: &mut HashSet<PathBuf>,
        complete: &mut HashSet<PathBuf>,
    ) {
        if complete.contains(&file.path) || !active.insert(file.path.clone()) {
            return;
        }
        for (_, target) in &file.imports {
            if let Some(target) = files.get(target) {
                self.bind_file(target, files, active, complete);
            }
        }
        let scope = self.file_scopes[&file.path];
        let tokens = self.tokens.get(&file.path).cloned().unwrap_or_default();
        for stmt in &file.statements {
            let Stmt::Import(spec) = stmt.unlocated() else {
                continue;
            };
            let Some((_, target)) = file.imports.iter().find(|(found, _)| {
                found.path == spec.path
                    && found.alias == spec.alias
                    && found
                        .selected
                        .iter()
                        .map(|name| (&name.name, &name.alias))
                        .eq(spec.selected.iter().map(|name| (&name.name, &name.alias)))
            }) else {
                continue;
            };
            let start = if let Stmt::Located { span, .. } = stmt {
                offset(&file.text, span.line, span.col)
            } else {
                0
            };
            let Some(path_token) = tokens
                .iter()
                .position(|token| token.start >= start && token.kind == K::String)
            else {
                continue;
            };
            if let Some(alias) = &spec.alias {
                if let Some(token) = tokens
                    .get(path_token + 2)
                    .filter(|token| token.lexeme == *alias)
                {
                    let id = self.add(
                        file,
                        token,
                        alias,
                        scope,
                        3,
                        String::new(),
                        None,
                        String::new(),
                        None,
                        true,
                    );
                    let key = format!("@module{}", self.by_id[&id]);
                    self.symbols.last_mut().unwrap().ty = key.clone();
                    self.module_aliases.insert(key, target.clone());
                }
            } else if !spec.selected.is_empty() {
                let mut cursor = path_token + 2;
                for selected in &spec.selected {
                    let Some(token) = tokens
                        .get(cursor)
                        .filter(|token| token.lexeme == selected.name)
                    else {
                        break;
                    };
                    cursor += 1;
                    let Some(index) = self
                        .exports
                        .get(target)
                        .and_then(|exports| exports.get(&selected.name))
                        .copied()
                    else {
                        continue;
                    };
                    let target_symbol = self.symbols[index].clone();
                    self.import_occurrence(
                        file,
                        token,
                        scope,
                        &target_symbol.id,
                        selected.alias.is_some(),
                    );
                    if let Some(alias) = &selected.alias {
                        cursor += 1; // contextual `sida`
                        if let Some(token) =
                            tokens.get(cursor).filter(|token| token.lexeme == *alias)
                        {
                            let id = self.add(
                                file,
                                token,
                                alias,
                                scope,
                                target_symbol.kind,
                                target_symbol.ty,
                                target_symbol.parameters,
                                target_symbol.returns,
                                None,
                                true,
                            );
                            self.alias_targets.insert(id, target_symbol.id);
                        }
                        cursor += 1;
                    } else {
                        self.by_name
                            .entry((scope, selected.name.clone()))
                            .or_default()
                            .push(index);
                    }
                    if tokens
                        .get(cursor)
                        .is_some_and(|token| token.kind == K::Comma)
                    {
                        cursor += 1;
                    }
                }
            } else if let Some(target_scope) = self.file_scopes.get(target) {
                // Flat imports intentionally retain the language's original
                // behavior, including private names and transitive imports.
                let bindings: Vec<_> = self
                    .by_name
                    .iter()
                    .filter(|((owner, _), _)| owner == target_scope)
                    .map(|((_, name), entries)| (name.clone(), entries.clone()))
                    .collect();
                for (name, entries) in bindings {
                    self.by_name
                        .entry((scope, name))
                        .or_default()
                        .extend(entries);
                }
            }
        }
        let explicit: Vec<_> = file
            .statements
            .iter()
            .filter_map(|stmt| {
                if let Stmt::Export(names) = stmt.unlocated() {
                    Some(names)
                } else {
                    None
                }
            })
            .flatten()
            .cloned()
            .collect();
        let has_explicit = file
            .statements
            .iter()
            .any(|stmt| matches!(stmt.unlocated(), Stmt::Export(_)));
        let mut exports = HashMap::new();
        for ((owner, name), entries) in &self.by_name {
            if *owner != scope
                || (has_explicit && !explicit.contains(name))
                || (!has_explicit && name.starts_with('_'))
            {
                continue;
            }
            if let Some(index) = entries
                .iter()
                .rev()
                .find(|index| self.symbols[**index].owner.is_none())
            {
                exports.insert(name.clone(), *index);
            }
        }
        self.exports.insert(file.path.clone(), exports);
        active.remove(&file.path);
        complete.insert(file.path.clone());
    }

    fn import_occurrence(
        &mut self,
        file: &SourceFile,
        token: &Token,
        scope: usize,
        id: &str,
        member: bool,
    ) {
        let location = Location {
            path: file.path.clone(),
            start: token.start,
            end: token.end,
        };
        if member {
            self.member_locations.insert(location.clone());
        }
        self.occurrences.push(Occurrence {
            location,
            symbol: id.into(),
            scope,
            declaration: false,
            write: false,
        });
    }

    pub(super) fn normalize_types(&mut self) {
        // Imports/reexports may introduce aliases to classes declared later in
        // the file set. Normalize only after the complete import graph is bound.
        let parents: Vec<_> = self
            .parents
            .iter()
            .filter_map(|(id, parent)| {
                let symbol = self.class_keys.get(id).and_then(|id| self.symbol(id))?;
                Some((
                    id.clone(),
                    self.canonical_type(
                        parent,
                        symbol.scope,
                        &symbol.location.path,
                        symbol.location.start,
                        0,
                    ),
                ))
            })
            .collect();
        self.parents = parents.into_iter().collect();
        for _ in 0..self.symbols.len().min(32) {
            let types: Vec<_> = self
                .symbols
                .iter()
                .map(|symbol| {
                    if let Some(target) = self
                        .alias_targets
                        .get(&symbol.id)
                        .and_then(|id| self.symbol(id))
                    {
                        return (target.ty.clone(), target.returns.clone());
                    }
                    let mut ty = self.canonical_type(
                        &symbol.ty,
                        symbol.scope,
                        &symbol.location.path,
                        symbol.location.start,
                        0,
                    );
                    if let Some(hints) = self.value_hints.get(&symbol.id) {
                        let types: Vec<_> = hints
                            .iter()
                            .map(|(expr, scope)| {
                                self.expression_type(
                                    expr,
                                    *scope,
                                    &symbol.location.path,
                                    symbol.location.start,
                                    0,
                                )
                            })
                            .collect();
                        if let Some(first) = types.first() {
                            ty = if types.iter().all(|ty| ty == first) {
                                first.clone()
                            } else {
                                "door".into()
                            };
                        }
                    }
                    (
                        ty,
                        self.canonical_type(
                            &symbol.returns,
                            symbol.scope,
                            &symbol.location.path,
                            symbol.location.start,
                            0,
                        ),
                    )
                })
                .collect();
            let mut changed = false;
            for (symbol, (ty, returns)) in self.symbols.iter_mut().zip(types) {
                changed |= symbol.ty != ty || symbol.returns != returns;
                symbol.ty = ty;
                symbol.returns = returns;
            }
            if !changed {
                break;
            }
        }
    }

    pub(super) fn canonical_type(
        &self,
        name: &str,
        scope: usize,
        path: &Path,
        at: usize,
        depth: usize,
    ) -> String {
        if depth > 32
            || self.by_id.contains_key(name)
            || self.class_keys.contains_key(name)
            || self.module_aliases.contains_key(name)
            || matches!(
                name,
                "door"
                    | "abn"
                    | "jajab"
                    | "qoraal"
                    | "bool"
                    | "baytyo"
                    | "maran"
                    | "teed"
                    | "walax"
                    | "hawl"
                    | "xirmo"
                    | "urur"
                    | ""
                    | "inferred"
            )
        {
            return name.into();
        }
        if let Some(base) = name.strip_suffix('?') {
            return format!("{}?", self.canonical_type(base, scope, path, at, depth + 1));
        }
        if let Some((container, inner)) = name
            .split_once('[')
            .filter(|(base, _)| matches!(*base, "teed" | "walax" | "urur" | "xirmo"))
        {
            if let Some(inner) = inner.strip_suffix(']') {
                let mut parts = Vec::new();
                let mut begin = 0;
                let mut nesting = 0usize;
                for (at_char, ch) in inner.char_indices() {
                    match ch {
                        '[' => nesting += 1,
                        ']' => nesting = nesting.saturating_sub(1),
                        ',' if nesting == 0 => {
                            parts.push(&inner[begin..at_char]);
                            begin = at_char + 1;
                        }
                        _ => (),
                    }
                }
                parts.push(&inner[begin..]);
                return format!(
                    "{container}[{}]",
                    parts
                        .into_iter()
                        .map(|part| self.canonical_type(part.trim(), scope, path, at, depth + 1))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        if name.contains(" | ") {
            return name
                .split(" | ")
                .map(|part| self.canonical_type(part, scope, path, at, depth + 1))
                .collect::<Vec<_>>()
                .join(" | ");
        }
        let mut names = name.split('.');
        let first = names.next().unwrap_or("");
        let Some(mut symbol) = self.resolve(first, scope, at, path) else {
            return name.into();
        };
        for member in names {
            let Some(found) = self.method(&symbol.ty, member) else {
                return name.into();
            };
            symbol = found;
        }
        if symbol.kind == 5 || self.module_aliases.contains_key(&symbol.ty) {
            symbol.ty.clone()
        } else {
            name.into()
        }
    }

    /// Definition follows explicit import aliases, while rename/references keep
    /// the local binding's own identity.
    pub fn definition_symbol<'a>(&'a self, symbol: &'a Symbol) -> &'a Symbol {
        let mut symbol = symbol;
        let mut seen = HashSet::new();
        while let Some(target) = self.alias_targets.get(&symbol.id) {
            if !seen.insert(target) {
                break;
            }
            let Some(found) = self.symbol(target) else {
                break;
            };
            symbol = found;
        }
        symbol
    }

    pub fn workspace_symbol(&self, id: &str) -> bool {
        self.symbol(id).is_some_and(|symbol| {
            symbol.owner.is_some()
                || self
                    .file_scopes
                    .values()
                    .any(|scope| *scope == symbol.scope)
        })
    }

    pub fn symbol_label(&self, symbol: &Symbol) -> String {
        if let Some(api) = self.native_api(&symbol.id) {
            return api.label(&api.signatures[0], None);
        }
        if let Some(parameters) = &symbol.parameters {
            let suffix = if symbol.returns == "door" {
                String::new()
            } else {
                format!(": {}", self.display_type(&symbol.returns))
            };
            format!("hawl {}({}){suffix}", symbol.name, parameters.join(", "))
        } else if symbol.kind == 5 {
            let constructor = self.method(&symbol.ty, "dhis");
            let parameters = constructor
                .and_then(|method| method.parameters.as_ref())
                .map(|parameters| parameters.join(", "))
                .unwrap_or_default();
            format!("qaab {}({parameters})", symbol.name)
        } else {
            format!("{}: {}", symbol.name, self.display_type(&symbol.ty))
        }
    }

    pub fn display_type(&self, ty: &str) -> String {
        if self.module_aliases.contains_key(ty) {
            return "keen".into();
        }
        let mut display = crate::native_api::display(ty);
        let mut keys: Vec<_> = self.class_keys.iter().collect();
        keys.sort_by_key(|(key, _)| std::cmp::Reverse(key.len()));
        for (key, id) in keys {
            if let Some(symbol) = self.symbol(id) {
                display = display.replace(key, &symbol.name);
            }
        }
        display
    }
}
