//! Conservative receiver inference for source navigation and completion.

use super::*;

impl Index {
    pub fn receiver_type(&self, path: &Path, start: usize, scope: usize) -> String {
        self.tokens
            .get(path)
            .and_then(|tokens| tokens.iter().position(|token| token.start == start))
            .map(|index| self.receiver_at(path, index, scope, 0))
            .unwrap_or_else(|| "door".into())
    }

    fn receiver_at(&self, path: &Path, index: usize, scope: usize, depth: usize) -> String {
        if depth > 32 {
            return "door".into();
        }
        let Some(tokens) = self.tokens.get(path) else {
            return "door".into();
        };
        let Some(token) = tokens.get(index) else {
            return "door".into();
        };
        if matches!(token.kind, K::Nafta | K::Waalid) {
            let mut cursor = Some(scope);
            while let Some(current) = cursor {
                if let Some(class) = &self.scopes[current].class {
                    return if token.kind == K::Waalid {
                        self.parents.get(class).cloned().unwrap_or_default()
                    } else {
                        class.clone()
                    };
                }
                cursor = self.scopes[current].parent;
            }
        }
        match token.kind {
            K::RParen | K::RBracket => {
                let open_kind = if token.kind == K::RParen {
                    K::LParen
                } else {
                    K::LBracket
                };
                let mut nesting = 0;
                let open = (0..index).rev().find(|position| {
                    if tokens[*position].kind == token.kind {
                        nesting += 1;
                    }
                    if tokens[*position].kind == open_kind {
                        if nesting == 0 {
                            return true;
                        }
                        nesting -= 1;
                    }
                    false
                });
                if let Some(open) = open.filter(|open| *open > 0) {
                    if token.kind == K::RBracket {
                        let ty = self.receiver_at(path, open - 1, scope, depth + 1);
                        return ty
                            .strip_prefix("teed[")
                            .and_then(|ty| ty.strip_suffix(']'))
                            .unwrap_or("door")
                            .into();
                    }
                    if let Some(symbol) = self.token_symbol(path, open - 1, scope, depth + 1) {
                        return if symbol.kind == 5 {
                            symbol.ty.clone()
                        } else {
                            symbol.returns.clone()
                        };
                    }
                    if index > open + 1 {
                        return self.receiver_at(path, index - 1, scope, depth + 1);
                    }
                }
                "door".into()
            }
            K::String | K::FormattedString => "qoraal".into(),
            K::Number => if token.lexeme.contains('.') {
                "jajab"
            } else {
                "abn"
            }
            .into(),
            _ => self
                .token_symbol(path, index, scope, depth + 1)
                .map(|symbol| symbol.ty.clone())
                .unwrap_or_else(|| "door".into()),
        }
    }

    fn token_symbol(
        &self,
        path: &Path,
        index: usize,
        scope: usize,
        depth: usize,
    ) -> Option<&Symbol> {
        if depth > 32 {
            return None;
        }
        let tokens = self.tokens.get(path)?;
        let token = tokens.get(index)?;
        if index >= 2 && tokens[index - 1].kind == K::Dot {
            let receiver = self.receiver_at(path, index - 2, scope, depth + 1);
            self.method(&receiver, &token.lexeme)
        } else {
            self.resolve(&token.lexeme, scope, token.start, path)
        }
    }

    pub(super) fn expression_type(
        &self,
        expr: &Expr,
        scope: usize,
        path: &Path,
        at: usize,
        depth: usize,
    ) -> String {
        if depth > 32 {
            return "door".into();
        }
        let at = if let Expr::Located { span, .. } = expr {
            self.tokens
                .get(path)
                .and_then(|tokens| {
                    tokens
                        .iter()
                        .find(|token| token.line == span.line && token.col == span.col)
                })
                .map(|token| token.start)
                .unwrap_or(at)
        } else {
            at
        };
        match expr.unlocated() {
            Expr::New { class_name, .. } => {
                self.canonical_type(class_name, scope, path, at, depth + 1)
            }
            Expr::Identifier(name) => self
                .resolve(name, scope, at, path)
                .map(|symbol| symbol.ty.clone())
                .unwrap_or_else(|| "door".into()),
            Expr::Call { name, .. } => self
                .resolve(name, scope, at, path)
                .map(|symbol| symbol.returns.clone())
                .unwrap_or_else(|| "door".into()),
            Expr::Property { obj, prop: name } => {
                let receiver = self.expression_type(obj, scope, path, at, depth + 1);
                self.method(&receiver, name)
                    .map(|symbol| symbol.ty.clone())
                    .unwrap_or_else(|| "door".into())
            }
            Expr::MethodCall { obj, method, .. } => {
                let receiver = self.expression_type(obj, scope, path, at, depth + 1);
                self.method(&receiver, method)
                    .map(|symbol| symbol.returns.clone())
                    .unwrap_or_else(|| "door".into())
            }
            Expr::List(values) if !values.is_empty() => {
                let first = self.expression_type(&values[0], scope, path, at, depth + 1);
                if values
                    .iter()
                    .skip(1)
                    .all(|value| self.expression_type(value, scope, path, at, depth + 1) == first)
                {
                    format!("teed[{first}]")
                } else {
                    "teed".into()
                }
            }
            Expr::Index { obj, .. } => {
                let receiver = self.expression_type(obj, scope, path, at, depth + 1);
                receiver
                    .strip_prefix("teed[")
                    .and_then(|ty| ty.strip_suffix(']'))
                    .unwrap_or("door")
                    .into()
            }
            _ => inferred(expr),
        }
    }

    /// Members include inherited methods and known instance fields; namespace
    /// completion exposes only the module's declared public exports.
    pub fn members_at(&self, path: &Path, at: usize) -> Option<Vec<&Symbol>> {
        let tokens = self.tokens.get(path)?;
        let mut cursor = tokens.partition_point(|token| token.start < at);
        if cursor > 0 && tokens[cursor - 1].kind == K::Eof {
            cursor -= 1;
        }
        if cursor > 0 && tokens[cursor - 1].kind == K::Identifier {
            cursor -= 1;
        }
        if cursor < 2 || tokens[cursor - 1].kind != K::Dot {
            return None;
        }
        let scope = self.scope_at(path, at);
        let mut ty = self.receiver_at(path, cursor - 2, scope, 0);
        let mut names = HashSet::new();
        let mut members = Vec::new();
        if let Some(module) = self.module_aliases.get(&ty) {
            for index in self.exports.get(module)?.values() {
                members.push(&self.symbols[*index]);
            }
        } else {
            let mut seen = HashSet::new();
            while seen.insert(ty.clone()) {
                for symbol in &self.symbols {
                    if symbol.owner.as_deref() == Some(&ty) && names.insert(symbol.name.clone()) {
                        members.push(symbol);
                    }
                }
                let Some(parent) = self.parents.get(&ty) else {
                    break;
                };
                ty = parent.clone();
            }
        }
        members.sort_by(|left, right| left.name.cmp(&right.name));
        Some(members)
    }
}
