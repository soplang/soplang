//! Native declarations share navigation identities with source declarations.
use super::*;

impl Index {
    pub(super) fn install_native(&mut self) {
        for api in crate::native_api::entries() {
            let location = Location {
                path: crate::native_api::SOURCE_PATH.into(),
                start: api.start,
                end: api.end,
            };
            let symbol = Symbol {
                id: api.id.clone(),
                name: api.name.clone(),
                location: location.clone(),
                kind: if api.receiver.is_some() { 6 } else { 12 },
                ty: "hawl".into(),
                scope: 0,
                parameters: Some(api.signatures[0].parameters.clone()),
                returns: api.result_for(api.receiver.as_deref().unwrap_or(""), None),
                owner: api.receiver.clone(),
                constant: true,
                parameter: false,
            };
            self.native.insert(api.id.clone(), api);
            self.by_id.insert(api.id.clone(), self.symbols.len());
            self.by_name
                .entry((0, api.name.clone()))
                .or_default()
                .push(self.symbols.len());
            self.occurrences.push(Occurrence {
                location,
                symbol: api.id.clone(),
                scope: 0,
                declaration: true,
                write: false,
            });
            self.symbols.push(symbol);
        }
    }
    pub fn native_api(&self, id: &str) -> Option<&'static crate::native_api::Api> {
        self.native.get(id).copied()
    }

    pub(super) fn call_result(
        &self,
        symbol: &Symbol,
        receiver: &str,
        count: Option<usize>,
    ) -> String {
        self.native_api(&symbol.id)
            .map(|api| api.result_for(receiver, count))
            .unwrap_or_else(|| symbol.returns.clone())
    }

    pub fn completion_receiver(&self, path: &Path, at: usize) -> Option<String> {
        let tokens = self.tokens.get(path)?;
        let mut cursor = tokens.partition_point(|token| token.start < at);
        if cursor > 0 && tokens[cursor - 1].kind == K::Eof {
            cursor -= 1;
        }
        if cursor > 0
            && (tokens[cursor - 1].kind == K::Identifier
                || cursor > 1 && tokens[cursor - 2].kind == K::Dot)
        {
            cursor -= 1;
        }
        if cursor < 2 || tokens[cursor - 1].kind != K::Dot {
            return None;
        }
        Some(self.receiver_at(path, cursor - 2, self.scope_at(path, at), 0))
    }

    pub fn member_receiver(&self, path: &Path, at: usize) -> Option<String> {
        let tokens = self.tokens.get(path)?;
        let index = tokens
            .iter()
            .position(|token| token.start <= at && at < token.end)?;
        if index >= 2 && tokens[index - 1].kind == K::Dot {
            Some(self.receiver_at(path, index - 2, self.scope_at(path, at), 0))
        } else {
            None
        }
    }

    /// Counts only the selected call's arguments. Tokenized strings (including
    /// interpolation) remain one argument at this nesting level.
    pub(super) fn argument_count(&self, path: &Path, open: usize, close: usize) -> usize {
        let Some(tokens) = self.tokens.get(path) else {
            return 0;
        };
        let mut depth = 0usize;
        let mut commas = 0;
        let mut content = false;
        let mut skip = 0;
        for token in &tokens[open + 1..close] {
            if token.start < skip {
                continue;
            }
            if matches!(token.kind, K::String | K::FormattedString) {
                skip = token.end;
            }
            match token.kind {
                K::LParen | K::LBracket | K::LBrace => depth += 1,
                K::RParen | K::RBracket | K::RBrace => depth = depth.saturating_sub(1),
                K::Comma if depth == 0 => commas += 1,
                K::Eof => continue,
                _ => {}
            }
            content = true;
        }
        usize::from(content) + commas
    }
}

impl Index {
    fn callable_origin<'a>(&'a self, symbol: &'a Symbol, depth: usize) -> Option<&'a Symbol> {
        if depth > 32 || self.reassigned.contains(&symbol.id) {
            return None;
        }
        if symbol.parameters.is_some() || symbol.kind == 5 {
            return Some(symbol);
        }
        let hints = self.value_hints.get(&symbol.id)?;
        let (expr, scope) = hints.last()?;
        let target = match expr.unlocated() {
            Expr::Identifier(name) => {
                self.resolve(name, *scope, symbol.location.start, &symbol.location.path)?
            }
            Expr::Property { obj, prop } => {
                let receiver = self.expression_type(
                    obj,
                    *scope,
                    &symbol.location.path,
                    symbol.location.start,
                    depth + 1,
                );
                // Native collection methods are direct calls, not stored fields.
                let target = self.method(&receiver, prop)?;
                if self.native_api(&target.id).is_some() {
                    return None;
                }
                target
            }
            _ => return None,
        };
        self.callable_origin(target, depth + 1)
    }

    /// Return the innermost callable whose argument list contains the cursor.
    /// Source is tokenized once; brackets, object literals and strings do not
    /// turn their commas into parameters of the surrounding call.
    pub fn call_site(
        &self,
        path: &Path,
        at: usize,
    ) -> Option<(Symbol, Option<String>, usize, usize, Option<String>)> {
        let tokens = self.tokens.get(path)?;
        let mut stack = Vec::new();
        let mut skip = 0;
        for (index, token) in tokens
            .iter()
            .enumerate()
            .take_while(|(_, token)| token.start < at)
        {
            if token.start < skip {
                continue;
            }
            if matches!(token.kind, K::String | K::FormattedString) && token.end <= at {
                skip = token.end;
            }
            match token.kind {
                K::LParen | K::LBracket | K::LBrace => stack.push(index),
                K::RParen | K::RBracket | K::RBrace => {
                    stack.pop();
                }
                _ => {}
            }
        }
        for open in stack
            .into_iter()
            .rev()
            .filter(|index| tokens[*index].kind == K::LParen && *index > 0)
        {
            // A function declaration's formal parameters are not call arguments.
            if open >= 2 && tokens[open - 2].kind == K::Hawl {
                continue;
            }
            let scope = self.scope_at(path, at);
            let Some(symbol) = self.token_symbol(path, open - 1, scope, 0) else {
                continue;
            };
            let Some(origin) = self.callable_origin(symbol, 0) else {
                continue;
            };
            let receiver = if open >= 3 && tokens[open - 2].kind == K::Dot {
                Some(self.receiver_at(path, open - 3, scope, 0))
            } else {
                None
            };
            let mut depth = 0usize;
            let mut active = 0;
            let mut count = 0;
            let mut current = Vec::new();
            let mut skip = 0;
            for token in tokens[open + 1..]
                .iter()
                .take_while(|token| token.start < at && token.kind != K::Eof)
            {
                if token.start < skip {
                    continue;
                }
                if matches!(token.kind, K::String | K::FormattedString) {
                    skip = token.end;
                }
                match token.kind {
                    K::LParen | K::LBracket | K::LBrace => depth += 1,
                    K::RParen | K::RBracket | K::RBrace => depth = depth.saturating_sub(1),
                    K::Comma if depth == 0 => {
                        active += 1;
                        current.clear();
                    }
                    _ => {}
                }
                if token.kind != K::Comma || depth != 0 {
                    current.push(token);
                }
                count = active + 1;
            }
            let name = (current.len() >= 2
                && current[0].kind == K::Identifier
                && current[1].kind == K::Colon)
                .then(|| current[0].lexeme.clone());
            return Some((origin.clone(), receiver, active, count, name));
        }
        None
    }
}
