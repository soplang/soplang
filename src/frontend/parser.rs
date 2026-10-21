//! Recursive descent parser: tokens → AST (Vec<Stmt>).
//! Parser for Soplang. IMPLEMENTATION_PLAN Phase 2.

use super::ast::{Expr, InterpolationPart, Literal, Param, Stmt, TypeAnnotation};
use super::token::{Token, TokenType};
use crate::error::{parser_error, SoplangError};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    contextual_keywords: std::collections::HashSet<usize>,
    source: Option<(Option<std::path::PathBuf>, Vec<String>)>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            contextual_keywords: std::collections::HashSet::new(),
            source: None,
        }
    }

    /// Keep locations for compiler execution; the plain parser API stays unwrapped.
    pub fn with_source(mut self, source: &str, path: Option<&std::path::Path>) -> Self {
        self.source = Some((
            path.map(std::path::Path::to_path_buf),
            source.split('\n').map(str::to_string).collect(),
        ));
        self
    }

    fn span(&self, token: &Token) -> Option<crate::error::SourceSpan> {
        let (file, lines) = self.source.as_ref()?;
        Some(crate::error::SourceSpan {
            file: Some(
                file.clone()
                    .unwrap_or_else(|| std::path::PathBuf::from("<command>")),
            ),
            line: token.line,
            col: token.col,
            end_col: {
                let mut bytes = 0;
                let count = lines
                    .get(token.line.saturating_sub(1))
                    .map(|line| {
                        line.trim_end_matches('\r')
                            .chars()
                            .skip(token.col.saturating_sub(1))
                            .take_while(|character| {
                                bytes += character.len_utf8();
                                bytes <= token.end.saturating_sub(token.start)
                            })
                            .count()
                    })
                    .unwrap_or(0);
                token.col + count.max(1) - 1
            },
            source_line: lines
                .get(token.line.saturating_sub(1))
                .cloned()
                .unwrap_or_default(),
        })
    }

    fn locate(&self, expr: Expr, token: &Token) -> Expr {
        match self.span(token) {
            Some(span) => Expr::Located {
                span,
                expr: Box::new(expr),
            },
            None => expr,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, SoplangError> {
        let mut stmts = Vec::new();
        while !self.at_end() {
            if self.check(TokenType::Semicolon) {
                self.advance();
                continue;
            }
            stmts.push(self.parse_stmt()?);
            self.consume_semicolons();
        }
        Ok(stmts)
    }

    /// Editor recovery: retain complete statements before the first syntax error.
    /// Normal compilation continues to use `parse`, with unchanged acceptance.
    pub fn parse_partial(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.at_end() {
            if self.check(TokenType::Semicolon) {
                self.advance();
                continue;
            }
            match self.parse_stmt() {
                Ok(stmt) => statements.push(stmt),
                Err(_) => break,
            }
            self.consume_semicolons();
        }
        statements
    }

    /// Parse the entire input as a single expression (for REPL / -c). Fails if there is extra input.
    pub fn parse_single_expression(&mut self) -> Result<Expr, SoplangError> {
        let e = self.parse_logical()?;
        self.consume_semicolons();
        if !self.at_end() {
            let t = self.peek();
            return Err(parser_error(
                format!(
                    "Waxaa la filayay hal weedh oo qudha, laakiin waxaa ka haray {}",
                    token_name_expected(&TokenType::Eof)
                ),
                t.line,
                t.col,
            ));
        }
        Ok(e)
    }

    /// Byte offsets where contextual identifiers were consumed as syntax.
    pub fn contextual_keyword_offsets(&self) -> &std::collections::HashSet<usize> {
        &self.contextual_keywords
    }

    fn advance_contextual(&mut self) {
        if self.peek().kind == TokenType::Identifier {
            self.contextual_keywords.insert(self.peek().start);
        }
        self.advance();
    }

    fn at_end(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }

    fn peek_nth(&self, n: usize) -> &Token {
        self.tokens
            .get(self.current + n)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    /// `ClassName var = ...` at the start of a statement or after `madoor`.
    fn looks_like_class_decl(&self) -> bool {
        matches!(self.peek().kind, TokenType::Identifier | TokenType::Null)
            && (matches!(
                self.peek_nth(1).kind,
                TokenType::Optional | TokenType::Union
            ) || (self.peek_nth(1).kind == TokenType::Identifier
                && self.peek_nth(2).kind == TokenType::Assign)
                || self.contextual_generic_declaration()
                || self.looks_like_dotted_type_decl())
    }

    fn looks_like_dotted_type_decl(&self) -> bool {
        if self.peek().kind != TokenType::Identifier {
            return false;
        }
        let mut offset = 0;
        while self.peek_nth(offset + 1).kind == TokenType::Dot
            && self.peek_nth(offset + 2).kind == TokenType::Identifier
        {
            offset += 2;
        }
        offset > 0
            && self.peek_nth(offset + 1).kind == TokenType::Identifier
            && self.peek_nth(offset + 2).kind == TokenType::Assign
    }

    fn contextual_generic_declaration(&self) -> bool {
        if !matches!(self.peek().lexeme.as_str(), "xirmo" | "urur")
            || self.peek_nth(1).kind != TokenType::LBracket
        {
            return false;
        }
        let mut depth = 0;
        let mut offset = 1;
        loop {
            match self.peek_nth(offset).kind {
                TokenType::LBracket => depth += 1,
                TokenType::RBracket => {
                    depth -= 1;
                    if depth == 0 {
                        offset += 1;
                        break;
                    }
                }
                TokenType::Identifier
                | TokenType::Abn
                | TokenType::Jajab
                | TokenType::Qoraal
                | TokenType::Bool
                | TokenType::Teed
                | TokenType::Walax
                | TokenType::Null
                | TokenType::Comma
                | TokenType::Optional
                | TokenType::Union => {}
                _ => return false,
            }
            offset += 1;
        }
        matches!(
            self.peek_nth(offset).kind,
            TokenType::Optional | TokenType::Union
        ) || (self.peek_nth(offset).kind == TokenType::Identifier
            && self.peek_nth(offset + 1).kind == TokenType::Assign)
    }

    fn starts_type_annotation(&self) -> bool {
        matches!(
            self.peek().kind,
            TokenType::Abn
                | TokenType::Jajab
                | TokenType::Qoraal
                | TokenType::Bool
                | TokenType::Teed
                | TokenType::Walax
                | TokenType::Null
        ) || (self.peek().kind == TokenType::Identifier
            && matches!(
                self.peek_nth(1).kind,
                TokenType::Identifier | TokenType::Optional | TokenType::Union
            ))
            || (self.peek().kind == TokenType::Identifier
                && matches!(self.peek().lexeme.as_str(), "xirmo" | "urur")
                && self.peek_nth(1).kind == TokenType::LBracket)
    }

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, SoplangError> {
        self.parse_type_annotation_depth(0, &mut 0)
    }

    fn parse_type_annotation_depth(
        &mut self,
        depth: usize,
        nodes: &mut usize,
    ) -> Result<TypeAnnotation, SoplangError> {
        if depth >= 32 {
            return Err(parser_error(
                "Noocyadu aad bay isugu dhex jiraan.",
                self.peek().line,
                self.peek().col,
            ));
        }
        let mut alternatives = Vec::new();
        loop {
            *nodes += 1;
            if *nodes > 1024 {
                return Err(parser_error(
                    "Qaabka noocan aad buu u weyn yahay.",
                    self.peek().line,
                    self.peek().col,
                ));
            }
            if alternatives.len() >= 32 {
                return Err(parser_error(
                    "Noocyada la isku daray aad bay u badan yihiin.",
                    self.peek().line,
                    self.peek().col,
                ));
            }
            let token = self.peek().clone();
            let mut ty = match token.kind {
                TokenType::Abn
                | TokenType::Jajab
                | TokenType::Qoraal
                | TokenType::Bool
                | TokenType::Teed
                | TokenType::Walax
                | TokenType::Null => token_to_type_ann(&token.kind),
                TokenType::Identifier => TypeAnnotation::named(token.lexeme.clone()),
                _ => {
                    return Err(parser_error(
                        "Waxaa la filayay nooc.",
                        token.line,
                        token.col,
                    ))
                }
            };
            self.advance();
            if let TypeAnnotation::Class(mut name) = ty {
                while self.check(TokenType::Dot) {
                    self.advance();
                    name.push('.');
                    name.push_str(&self.expect_identifier()?);
                }
                ty = TypeAnnotation::Class(name);
            }
            if self.check(TokenType::LBracket) {
                self.advance();
                if ty == TypeAnnotation::Tuple {
                    let mut types = Vec::new();
                    if !self.check(TokenType::RBracket) {
                        loop {
                            types.push(self.parse_type_annotation_depth(depth + 1, nodes)?);
                            if !self.check(TokenType::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }
                    ty = TypeAnnotation::TupleOf(types);
                } else {
                    let inner_token = self.peek().clone();
                    let inner = self.parse_type_annotation_depth(depth + 1, nodes)?;
                    ty = match ty {
                        TypeAnnotation::Teed => TypeAnnotation::ListOf(Box::new(inner)),
                        TypeAnnotation::Set => TypeAnnotation::SetOf(Box::new(inner)),
                        TypeAnnotation::Walax => {
                            let value = if self.check(TokenType::Comma) {
                                if inner != TypeAnnotation::Qoraal {
                                    return Err(parser_error(
                                        "Furaha walaxda noocaysan waa inuu noqdaa qoraal.",
                                        inner_token.line,
                                        inner_token.col,
                                    ));
                                }
                                self.advance();
                                self.parse_type_annotation_depth(depth + 1, nodes)?
                            } else {
                                inner
                            };
                            TypeAnnotation::ObjectOf(Box::new(value))
                        }
                        _ => {
                            return Err(parser_error(
                                "Noocan ma qaato noocyo xubno ah.",
                                token.line,
                                token.col,
                            ))
                        }
                    };
                }
                self.expect(TokenType::RBracket)?;
            }
            if self.check(TokenType::Optional) {
                self.advance();
                ty = TypeAnnotation::union(vec![ty, TypeAnnotation::Null]);
            }
            alternatives.push(ty);
            if !self.check(TokenType::Union) {
                break;
            }
            self.advance();
        }
        Ok(TypeAnnotation::union(alternatives))
    }

    /// Recognize a contextual branch without consuming an ordinary `kale(...)`
    /// call following an if statement. A conditional header must end in `{`.
    fn looks_like_kale_branch(&self) -> bool {
        if self.peek().kind != TokenType::Identifier || self.peek().lexeme != "kale" {
            return false;
        }
        if self.peek_nth(1).kind == TokenType::LBrace {
            return true;
        }
        if self.peek_nth(1).kind != TokenType::LParen {
            return false;
        }
        let mut depth = 1usize;
        for index in self.current + 2..self.tokens.len() {
            match self.tokens[index].kind {
                TokenType::LParen => depth += 1,
                TokenType::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return self.tokens.get(index + 1).map(|token| &token.kind)
                            == Some(&TokenType::LBrace);
                    }
                }
                TokenType::Eof => break,
                _ => {}
            }
        }
        false
    }

    fn peek(&self) -> &Token {
        if self.current >= self.tokens.len() {
            return self.tokens.last().unwrap();
        }
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        if self.current < self.tokens.len() && self.tokens[self.current].kind != TokenType::Eof {
            self.current += 1;
        }
        self.tokens
            .get(self.current.saturating_sub(1))
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn check(&self, kind: TokenType) -> bool {
        !self.at_end() && std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&kind)
    }

    fn consume_semicolons(&mut self) {
        while self.check(TokenType::Semicolon) {
            self.advance();
        }
    }

    fn expect(&mut self, kind: TokenType) -> Result<&Token, SoplangError> {
        let t = self.peek();
        if std::mem::discriminant(&t.kind) != std::mem::discriminant(&kind) {
            return Err(parser_error(
                format!(
                    "Waxaa la filayay {}, laakiin waxaa la helay {:?}",
                    token_name_expected(&kind),
                    t.lexeme
                ),
                t.line,
                t.col,
            ));
        }
        Ok(self.advance())
    }

    fn parse_stmt(&mut self) -> Result<Stmt, SoplangError> {
        let token = self.peek().clone();
        let stmt = self.parse_stmt_inner()?;
        Ok(match self.span(&token) {
            Some(span) => Stmt::Located {
                span,
                stmt: Box::new(stmt),
            },
            None => stmt,
        })
    }

    fn parse_stmt_inner(&mut self) -> Result<Stmt, SoplangError> {
        let t = self.peek();
        let (line, col) = (t.line, t.col);

        // Contextual spelling: existing variables and functions named tijaabi
        // remain valid; only `tijaabi {` starts a protected block.
        if t.kind == TokenType::Identifier
            && t.lexeme == "tijaabi"
            && self.tokens.get(self.current + 1).map(|t| &t.kind) == Some(&TokenType::LBrace)
        {
            return self.parse_try_catch(line, col);
        }
        if t.kind == TokenType::Identifier
            && t.lexeme == "tijaabo"
            && self.peek_nth(1).kind == TokenType::String
        {
            return self.parse_test();
        }
        if t.kind == TokenType::Identifier
            && matches!(t.lexeme.as_str(), "diyaar" | "nadiifi")
            && self.peek_nth(1).kind == TokenType::LBrace
        {
            return self.parse_fixture(t.lexeme == "diyaar");
        }

        // Calls, assignments and property access on an identifier named tuur
        // retain their meaning. Raising requires an adjacent same-line value.
        if t.kind == TokenType::Identifier
            && t.lexeme == "tuur"
            && self.tokens.get(self.current + 1).is_some_and(|next| {
                next.line == line
                    && matches!(
                        next.kind,
                        TokenType::String
                            | TokenType::FormattedString
                            | TokenType::Number
                            | TokenType::Identifier
                            | TokenType::True
                            | TokenType::False
                            | TokenType::Null
                            | TokenType::Cusub
                            | TokenType::Hawl
                            | TokenType::LBrace
                    )
            })
        {
            self.advance();
            return Ok(Stmt::Throw(self.parse_expression()?));
        }

        if t.kind == TokenType::Null && self.looks_like_class_decl() {
            let ann = self.parse_type_annotation()?;
            return self.parse_var_decl_rest(ann, false, line, col);
        }
        match &t.kind {
            TokenType::Haddii => return self.parse_if(line, col),
            TokenType::Dooro => return self.parse_switch(line, col),
            TokenType::Door => return self.parse_var_decl(false, false, line, col),
            TokenType::Madoor => return self.parse_madoor(line, col),
            TokenType::Hawl if self.peek_nth(1).kind == TokenType::LParen => {
                return Ok(Stmt::Expr(self.parse_logical()?));
            }
            TokenType::Hawl => return self.parse_func_def(),
            TokenType::Celi => return self.parse_return(),
            TokenType::Qor => return self.parse_qor_statement(line, col),
            TokenType::Gelin => return self.parse_function_call_stmt(line, col),
            TokenType::Kuceli => return self.parse_for(line, col),
            TokenType::Intay => return self.parse_while(line, col),
            TokenType::Jooji => {
                self.advance();
                return Ok(Stmt::Break);
            }
            TokenType::Soco => {
                self.advance();
                return Ok(Stmt::Continue);
            }
            TokenType::Fasax => return self.parse_try_catch(line, col),
            TokenType::Keen => return self.parse_import(),
            TokenType::Identifier
                if t.lexeme == "soosaar" && self.peek_nth(1).kind == TokenType::Identifier =>
            {
                return self.parse_export();
            }
            TokenType::Qaab => return self.parse_class_def(),
            TokenType::LParen
            | TokenType::LBracket
            | TokenType::Number
            | TokenType::String
            | TokenType::FormattedString
            | TokenType::True
            | TokenType::False
            | TokenType::Null => {
                return Ok(Stmt::Expr(self.parse_logical()?));
            }
            TokenType::LBrace => return self.parse_block(),
            TokenType::Identifier if self.looks_like_class_decl() => {
                let ann = self.parse_type_annotation()?;
                return self.parse_var_decl_rest(ann, false, line, col);
            }
            TokenType::Identifier | TokenType::Nafta | TokenType::Waalid => {
                return self.parse_identifier_stmt(line, col)
            }
            _ => {}
        }

        if self.peek_nth(1).kind == TokenType::LParen
            && matches!(
                t.kind,
                TokenType::Abn
                    | TokenType::Jajab
                    | TokenType::Qoraal
                    | TokenType::Bool
                    | TokenType::Teed
                    | TokenType::Walax
            )
        {
            return Ok(Stmt::Expr(self.parse_logical()?));
        }

        // Static type keywords
        if matches!(
            t.kind,
            TokenType::Abn
                | TokenType::Jajab
                | TokenType::Qoraal
                | TokenType::Bool
                | TokenType::Teed
                | TokenType::Walax
        ) {
            return self.parse_var_decl_static(line, col);
        }

        if matches!(t.kind, TokenType::HaddiiKale | TokenType::Ugudambeyn) {
            return Err(parser_error(
                "haddii_kale iyo ugudambeyn waa in ay raacaan 'haddii'",
                line,
                col,
            ));
        }

        Err(parser_error(
            format!("Calaamad aan la filayn: {:?}", t.lexeme),
            line,
            col,
        ))
    }

    fn parse_madoor(&mut self, line: usize, col: usize) -> Result<Stmt, SoplangError> {
        self.advance(); // madoor
        let type_ann = if self.starts_type_annotation() {
            self.parse_type_annotation()?
        } else {
            TypeAnnotation::Dynamic
        };
        self.parse_var_decl_rest(type_ann, true, line, col)
    }

    fn parse_var_decl_static(&mut self, line: usize, col: usize) -> Result<Stmt, SoplangError> {
        let type_ann = self.parse_type_annotation()?;
        self.parse_var_decl_rest(type_ann, false, line, col)
    }

    fn parse_var_decl(
        &mut self,
        _is_static: bool,
        is_const: bool,
        line: usize,
        col: usize,
    ) -> Result<Stmt, SoplangError> {
        self.advance(); // door
        self.parse_var_decl_rest(TypeAnnotation::Dynamic, is_const, line, col)
    }

    fn parse_var_decl_rest(
        &mut self,
        type_ann: TypeAnnotation,
        is_const: bool,
        line: usize,
        col: usize,
    ) -> Result<Stmt, SoplangError> {
        let name = self.expect_identifier()?;
        self.expect(TokenType::Assign)?;
        let value = self.parse_logical()?;
        Ok(Stmt::VarDecl {
            name,
            type_ann,
            is_const,
            value,
            line,
            col,
        })
    }

    fn current_token_call_name(&self) -> String {
        if matches!(self.peek().kind, TokenType::Identifier) {
            self.peek().lexeme.clone()
        } else {
            token_type_to_name(&self.peek().kind)
        }
    }

    fn expect_identifier(&mut self) -> Result<String, SoplangError> {
        let t = self.peek();
        match &t.kind {
            TokenType::Identifier => {
                let s = t.lexeme.clone();
                self.advance();
                Ok(s)
            }
            TokenType::Nafta => {
                self.advance();
                Ok("nafta".to_string())
            }
            _ => Err(parser_error(
                format!(
                    "Waxaa la filayay magac doorsame, laakiin waxaa la helay {:?}",
                    t.lexeme
                ),
                t.line,
                t.col,
            )),
        }
    }

    fn expect_member_name(&mut self) -> Result<String, SoplangError> {
        if matches!(
            self.peek().kind,
            TokenType::Kuceli | TokenType::Teed | TokenType::Qoraal
        ) {
            return Ok(self.advance().lexeme.clone());
        }
        self.expect_identifier()
    }

    fn parse_func_def(&mut self) -> Result<Stmt, SoplangError> {
        self.advance(); // hawl
        let name = self.expect_identifier()?;
        let (params, return_ann, body) = self.parse_function_tail()?;
        Ok(Stmt::FuncDef {
            name,
            params,
            return_ann,
            body,
        })
    }

    fn next_keyword_collector(&self) -> bool {
        let mut offset = 0;
        if self.check(TokenType::Walax) {
            offset = 1;
            if self.peek_nth(offset).kind == TokenType::LBracket {
                let mut depth = 0;
                loop {
                    match self.peek_nth(offset).kind {
                        TokenType::LBracket => depth += 1,
                        TokenType::RBracket => {
                            depth -= 1;
                            if depth == 0 {
                                offset += 1;
                                break;
                            }
                        }
                        TokenType::Eof => return false,
                        _ => {}
                    }
                    offset += 1;
                }
            }
        }
        self.peek_nth(offset).kind == TokenType::Star
            && self.peek_nth(offset + 1).kind == TokenType::Star
    }

    fn parse_function_tail(
        &mut self,
    ) -> Result<(Vec<Param>, TypeAnnotation, Vec<Stmt>), SoplangError> {
        self.expect(TokenType::LParen)?;
        let mut params = Vec::new();
        let mut names = std::collections::HashSet::new();
        let mut optional = false;
        let mut saw_rest = false;
        while !self.check(TokenType::RParen) {
            let mut type_ann = if self.starts_type_annotation() {
                self.parse_type_annotation()?
            } else {
                TypeAnnotation::Dynamic
            };
            let rest_marker = self.peek().clone();
            let starred = self.check(TokenType::Star);
            if starred {
                self.advance();
            }
            let keyword_rest = starred && self.check(TokenType::Star);
            if keyword_rest {
                self.advance();
            }
            let variadic = starred && !keyword_rest;
            if saw_rest && !keyword_rest {
                return Err(parser_error(
                    "Doodda '*' waxaa ka dambayn karta '**' oo keliya.",
                    rest_marker.line,
                    rest_marker.col,
                ));
            }
            if starred {
                let expected = if keyword_rest {
                    TypeAnnotation::Walax
                } else {
                    TypeAnnotation::Teed
                };
                let typed_collection = matches!(
                    (&type_ann, &expected),
                    (TypeAnnotation::ListOf(_), TypeAnnotation::Teed)
                        | (TypeAnnotation::ObjectOf(_), TypeAnnotation::Walax)
                );
                if type_ann != TypeAnnotation::Dynamic && type_ann != expected && !typed_collection
                {
                    return Err(parser_error(
                        if keyword_rest {
                            "Doodda '**' waxay ururisaa walax; nooc kale lama siin karo."
                        } else {
                            "Doodda '*' waxay ururisaa teed; nooc kale lama siin karo."
                        },
                        rest_marker.line,
                        rest_marker.col,
                    ));
                }
                if type_ann == TypeAnnotation::Dynamic {
                    type_ann = expected;
                }
                saw_rest = true;
            }
            let marker = self.peek().clone();
            let name = self.expect_identifier()?;
            if !names.insert(name.clone()) {
                return Err(parser_error(
                    format!("Doodda '{}' mar hore ayaa lagu qeexay hawshan.", name),
                    marker.line,
                    marker.col,
                ));
            }
            let default = if self.check(TokenType::Assign) {
                if starred {
                    return Err(parser_error(
                        if keyword_rest {
                            "Doodda '**' ma yeelan karto qiime hordhac ah."
                        } else {
                            "Doodda '*' ma yeelan karto qiime hordhac ah."
                        },
                        marker.line,
                        marker.col,
                    ));
                }
                self.advance();
                optional = true;
                Some(self.parse_logical()?)
            } else {
                if optional && !starred {
                    return Err(parser_error(
                        "Dood aan lahayn qiime hordhac ah ma raaci karto dood leh qiime hordhac ah.",
                        marker.line,
                        marker.col,
                    ));
                }
                None
            };
            params.push(Param {
                name,
                type_ann,
                default,
                variadic,
                keyword_rest,
            });
            if !self.check(TokenType::RParen) {
                self.expect(TokenType::Comma)?;
            }
            let next_keyword_collector = self.next_keyword_collector();
            if (keyword_rest || (variadic && !next_keyword_collector))
                && !self.check(TokenType::RParen)
            {
                return Err(parser_error(
                    if keyword_rest {
                        "Doodda '**' waa inay noqotaa doodda ugu dambaysa."
                    } else {
                        "Doodda '*' waa inay noqotaa doodda ugu dambaysa."
                    },
                    self.peek().line,
                    self.peek().col,
                ));
            }
        }
        self.expect(TokenType::RParen)?;
        let return_ann = if self.check(TokenType::Colon) {
            self.advance();
            self.parse_type_annotation()?
        } else {
            TypeAnnotation::Dynamic
        };
        self.expect(TokenType::LBrace)?;
        let body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        Ok((params, return_ann, body))
    }

    fn parse_anonymous_function(&mut self) -> Result<Expr, SoplangError> {
        self.advance(); // hawl
        let (params, return_ann, body) = self.parse_function_tail()?;
        Ok(Expr::AnonymousFunc {
            params,
            return_ann,
            body,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, SoplangError> {
        self.advance(); // celi
        if self.check(TokenType::Semicolon) || self.at_end() || self.check(TokenType::RBrace) {
            return Ok(Stmt::Return(None));
        }
        let e = self.parse_logical()?;
        Ok(Stmt::Return(Some(e)))
    }

    fn parse_qor_statement(&mut self, _line: usize, _col: usize) -> Result<Stmt, SoplangError> {
        if self.peek_nth(1).kind == TokenType::LParen {
            return self.parse_function_call_stmt(_line, _col);
        }
        self.advance(); // qor
        let arg = self.parse_logical()?;
        Ok(Stmt::Expr(Expr::Call {
            name: "qor".to_string(),
            args: vec![arg],
        }))
    }

    fn parse_function_call_stmt(
        &mut self,
        _line: usize,
        _col: usize,
    ) -> Result<Stmt, SoplangError> {
        Ok(Stmt::Expr(self.parse_logical()?))
    }

    fn parse_if(&mut self, _line: usize, _col: usize) -> Result<Stmt, SoplangError> {
        self.advance_contextual(); // haddii
        self.expect(TokenType::LParen)?;
        let cond = self.parse_logical()?;
        self.expect(TokenType::RParen)?;
        self.expect(TokenType::LBrace)?;
        let then_body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;

        let mut elseifs = Vec::new();
        while self.check(TokenType::HaddiiKale)
            || (self.looks_like_kale_branch() && self.peek_nth(1).kind == TokenType::LParen)
        {
            self.advance_contextual();
            self.expect(TokenType::LParen)?;
            let elif_cond = self.parse_logical()?;
            self.expect(TokenType::RParen)?;
            self.expect(TokenType::LBrace)?;
            let elif_body = self.parse_block_stmts()?;
            self.expect(TokenType::RBrace)?;
            elseifs.push((elif_cond, elif_body));
        }

        let else_body = if self.check(TokenType::Ugudambeyn)
            || (self.looks_like_kale_branch() && self.peek_nth(1).kind == TokenType::LBrace)
        {
            self.advance_contextual();
            self.expect(TokenType::LBrace)?;
            let b = self.parse_block_stmts()?;
            self.expect(TokenType::RBrace)?;
            Some(b)
        } else {
            None
        };

        Ok(Stmt::If {
            cond,
            then_body,
            elseifs,
            else_body,
        })
    }

    fn parse_switch(&mut self, _line: usize, _col: usize) -> Result<Stmt, SoplangError> {
        self.advance_contextual(); // dooro
        self.expect(TokenType::LParen)?;
        let expr = self.parse_logical()?;
        self.expect(TokenType::RParen)?;
        self.expect(TokenType::LBrace)?;

        let mut cases = Vec::new();
        let mut default = None;

        while !self.check(TokenType::RBrace) {
            if self.check(TokenType::Xaalad) {
                self.advance_contextual();
                let case_val = self.parse_logical()?;
                self.expect(TokenType::LBrace)?;
                let body = self.parse_block_stmts()?;
                self.expect(TokenType::RBrace)?;
                cases.push((case_val, body));
            } else if self.check(TokenType::Ugudambeyn)
                || (self.looks_like_kale_branch() && self.peek_nth(1).kind == TokenType::LBrace)
            {
                self.advance_contextual();
                self.expect(TokenType::LBrace)?;
                default = Some(self.parse_block_stmts()?);
                self.expect(TokenType::RBrace)?;
            } else {
                let t = self.peek();
                return Err(parser_error(
                    "Waxaa la filayay 'xaalad' ama 'kale' (ama 'ugudambeyn')",
                    t.line,
                    t.col,
                ));
            }
        }
        self.expect(TokenType::RBrace)?;
        Ok(Stmt::Switch {
            expr,
            cases,
            default,
        })
    }

    fn parse_for(&mut self, _line: usize, _col: usize) -> Result<Stmt, SoplangError> {
        self.advance_contextual(); // kuceli
        self.expect(TokenType::LParen)?;
        let var = self.expect_identifier()?;
        if self.check(TokenType::Comma) {
            let mut vars = vec![var];
            let mut names = std::collections::HashSet::from([vars[0].clone()]);
            while self.check(TokenType::Comma) {
                self.advance();
                let token = self.peek().clone();
                let name = self.expect_identifier()?;
                if !names.insert(name.clone()) {
                    return Err(parser_error(
                        "Magacyada wareeggu waa inay kala duwanaadaan.",
                        token.line,
                        token.col,
                    ));
                }
                vars.push(name);
            }
            if self.peek().kind != TokenType::Identifier || self.peek().lexeme != "ku" {
                return Err(parser_error(
                    "Waxaa la filayay 'ku' (kuceli).",
                    self.peek().line,
                    self.peek().col,
                ));
            }
            self.advance_contextual();
            let iterable = self.parse_expression()?;
            return self.finish_for_each(vars, iterable);
        }
        if self.peek().kind == TokenType::Identifier && self.peek().lexeme == "ku" {
            // Preserve every complete numeric header starting with a variable
            // or call named ku. A parenthesized iterable disambiguates the rare
            // source expression that is itself a complete numeric header.
            let saved = self.current;
            let offset = self.peek().start;
            if self.has_range_separator() {
                if let Ok((start, end, step)) = self.parse_range_header() {
                    return self.finish_range_for(var, start, end, step);
                }
            }
            self.current = saved;
            // Header parsing only moves forward; discard its contextual marks.
            self.contextual_keywords
                .retain(|position| *position < offset);
            self.advance_contextual();
            let iterable = self.parse_expression()?;
            return self.finish_for_each(vec![var], iterable);
        }
        let (start, end, step) = self.parse_range_header()?;
        self.finish_range_for(var, start, end, step)
    }

    fn has_range_separator(&self) -> bool {
        // Do not speculatively parse nested function bodies in foreach sources.
        // A numeric header always has an ilaa outside its nested expressions.
        let mut depth = 0usize;
        for token in &self.tokens[self.current..] {
            match token.kind {
                TokenType::RParen if depth == 0 => break,
                TokenType::Identifier if depth == 0 && token.lexeme == "ilaa" => return true,
                TokenType::LParen | TokenType::LBracket | TokenType::LBrace => depth += 1,
                TokenType::RParen | TokenType::RBracket | TokenType::RBrace => {
                    depth = depth.saturating_sub(1)
                }
                TokenType::Eof => break,
                _ => (),
            }
        }
        false
    }

    fn parse_range_header(&mut self) -> Result<(Expr, Expr, Option<Expr>), SoplangError> {
        let start = self.parse_expression()?;
        // "ilaa" keyword
        let t = self.peek();
        if t.kind != TokenType::Identifier || t.lexeme != "ilaa" {
            return Err(parser_error(
                "Waxaa la filayay 'ilaa' (kuceli)",
                t.line,
                t.col,
            ));
        }
        self.advance_contextual();
        let end = self.parse_expression()?;
        let step = if self.check(TokenType::Colon) {
            self.advance_contextual();
            if self.check(TokenType::Colon) {
                self.advance_contextual();
                Some(self.parse_expression()?)
            } else {
                return Err(parser_error(
                    "Waxaa la filayay '::' (tallaabo)",
                    self.peek().line,
                    self.peek().col,
                ));
            }
        } else {
            None
        };
        self.expect(TokenType::RParen)?;
        Ok((start, end, step))
    }

    fn finish_range_for(
        &mut self,
        var: String,
        start: Expr,
        end: Expr,
        step: Option<Expr>,
    ) -> Result<Stmt, SoplangError> {
        self.expect(TokenType::LBrace)?;
        let body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        Ok(Stmt::For {
            var,
            start,
            end,
            step,
            body,
        })
    }

    fn finish_for_each(&mut self, vars: Vec<String>, iterable: Expr) -> Result<Stmt, SoplangError> {
        self.expect(TokenType::RParen)?;
        self.expect(TokenType::LBrace)?;
        let body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        Ok(Stmt::ForEach {
            vars,
            iterable,
            body,
        })
    }

    fn parse_while(&mut self, _line: usize, _col: usize) -> Result<Stmt, SoplangError> {
        self.advance(); // intay
        self.expect(TokenType::LParen)?;
        let cond = self.parse_logical()?;
        self.expect(TokenType::RParen)?;
        self.expect(TokenType::LBrace)?;
        let body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_try_catch(&mut self, _line: usize, _col: usize) -> Result<Stmt, SoplangError> {
        self.advance_contextual(); // tijaabi or fasax
        self.expect(TokenType::LBrace)?;
        let try_body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        let (err_var, err_details, catch_body) = if self.check(TokenType::Qabo) {
            self.advance();
            self.expect(TokenType::LParen)?;
            let name = self.expect_identifier()?;
            let details = if self.check(TokenType::Comma) {
                self.advance();
                let details = self.expect_identifier()?;
                if details == name {
                    return Err(parser_error(
                        "Magacyada 'qabo' waa inay kala duwanaadaan.",
                        self.peek().line,
                        self.peek().col,
                    ));
                }
                Some(details)
            } else {
                None
            };
            self.expect(TokenType::RParen)?;
            self.expect(TokenType::LBrace)?;
            let body = self.parse_block_stmts()?;
            self.expect(TokenType::RBrace)?;
            (Some(name), details, body)
        } else {
            (None, None, Vec::new())
        };
        let has_cleanup = self.peek().kind == TokenType::Identifier
            && self.peek().lexeme == "dhammee"
            && self.tokens.get(self.current + 1).map(|token| &token.kind)
                == Some(&TokenType::LBrace);
        let finally_body = if has_cleanup {
            self.advance();
            self.expect(TokenType::LBrace)?;
            let body = self.parse_block_stmts()?;
            self.expect(TokenType::RBrace)?;
            body
        } else {
            Vec::new()
        };
        if err_var.is_none() && !has_cleanup {
            return Err(parser_error(
                "Waxaa la filayay 'qabo' ama 'dhammee' ka dib 'tijaabi'.",
                self.peek().line,
                self.peek().col,
            ));
        }
        Ok(Stmt::TryCatch {
            try_body,
            err_var,
            err_details,
            catch_body,
            finally_body,
        })
    }

    fn parse_import(&mut self) -> Result<Stmt, SoplangError> {
        self.advance(); // keen
        let t = self.peek();
        let path = match &t.kind {
            TokenType::String => {
                let s = t.lexeme.clone();
                self.advance();
                s
            }
            _ => {
                return Err(parser_error(
                    "Waxaa la filayay qoraal ah jidka faylka",
                    t.line,
                    t.col,
                ));
            }
        };
        let alias = if self.peek().kind == TokenType::Identifier && self.peek().lexeme == "sida" {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        if alias.is_some()
            && self.peek().kind == TokenType::Identifier
            && self.peek().lexeme == "ka"
        {
            return Err(parser_error(
                "Ha isku darin sida iyo ka isla hal keen.",
                self.peek().line,
                self.peek().col,
            ));
        }
        let selected = if alias.is_none()
            && self.peek().kind == TokenType::Identifier
            && self.peek().lexeme == "ka"
        {
            self.advance();
            self.parse_selected_imports()?
        } else {
            Vec::new()
        };
        Ok(Stmt::Import(crate::frontend::ast::ImportSpec {
            path,
            alias,
            selected,
        }))
    }

    fn parse_selected_imports(
        &mut self,
    ) -> Result<Vec<crate::frontend::ast::SelectedImport>, SoplangError> {
        let mut items = Vec::new();
        loop {
            let name = self.expect_identifier()?;
            let alias = if self.peek().kind == TokenType::Identifier && self.peek().lexeme == "sida"
            {
                self.advance();
                Some(self.expect_identifier()?)
            } else {
                None
            };
            items.push(crate::frontend::ast::SelectedImport { name, alias });
            if self.check(TokenType::Comma) {
                self.advance();
                continue;
            }
            break;
        }
        Ok(items)
    }

    fn parse_export(&mut self) -> Result<Stmt, SoplangError> {
        self.advance();
        let mut names = vec![self.expect_identifier()?];
        while self.check(TokenType::Comma) {
            self.advance();
            names.push(self.expect_identifier()?);
        }
        Ok(Stmt::Export(names))
    }

    fn parse_dotted_name(&mut self) -> Result<String, SoplangError> {
        let mut name = self.expect_identifier()?;
        while self.check(TokenType::Dot) {
            self.advance();
            name.push('.');
            name.push_str(&self.expect_identifier()?);
        }
        Ok(name)
    }

    fn parse_class_def(&mut self) -> Result<Stmt, SoplangError> {
        let t = self.peek();
        let (line, col) = (t.line, t.col);
        self.advance(); // qaab
        let name = self.expect_identifier()?;
        let parent = if self.check(TokenType::Dhaxal) {
            self.advance();
            Some(self.parse_dotted_name()?)
        } else {
            None
        };
        self.expect(TokenType::LBrace)?;
        let body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        for s in &body {
            if !matches!(s.unlocated(), Stmt::FuncDef { .. }) {
                let error = parser_error("Jirka qaabka waa in uu ka koobnaado hawlo kaliya", 0, 0);
                return Err(match s {
                    Stmt::Located { span, .. } => span.attach(error),
                    _ => parser_error(
                        "Jirka qaabka waa in uu ka koobnaado hawlo kaliya",
                        line,
                        col,
                    ),
                });
            }
        }
        Ok(Stmt::ClassDef { name, parent, body })
    }

    fn parse_block(&mut self) -> Result<Stmt, SoplangError> {
        self.expect(TokenType::LBrace)?;
        let stmts = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        Ok(Stmt::Block(stmts))
    }

    fn parse_block_stmts(&mut self) -> Result<Vec<Stmt>, SoplangError> {
        let mut stmts = Vec::new();
        while !self.check(TokenType::RBrace) && !self.at_end() {
            if self.check(TokenType::Semicolon) {
                self.advance();
                continue;
            }
            stmts.push(self.parse_stmt()?);
            self.consume_semicolons();
        }
        Ok(stmts)
    }

    fn parse_identifier_stmt(&mut self, line: usize, col: usize) -> Result<Stmt, SoplangError> {
        let left = self.parse_logical()?;
        if self.check(TokenType::Assign) {
            if !matches!(
                left.unlocated(),
                Expr::Identifier(_) | Expr::Index { .. } | Expr::Property { .. }
            ) {
                return Err(parser_error(
                    "Meesha qiime lagu qorayo waa inay noqotaa doorsame, tusmo ama sifo.",
                    line,
                    col,
                ));
            }
            self.advance();
            let value = self.parse_logical()?;
            return Ok(Stmt::Assign {
                target: left,
                value,
                line,
                col,
            });
        }
        Ok(Stmt::Expr(left))
    }

    fn parse_call_rest(&mut self, name: String) -> Result<Expr, SoplangError> {
        self.expect(TokenType::LParen)?;
        let args = self.parse_call_args()?;
        Ok(Expr::Call { name, args })
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, SoplangError> {
        let mut args = Vec::new();
        let mut names = std::collections::HashSet::new();
        let mut named = false;
        while !self.check(TokenType::RParen) {
            let marker = self.peek().clone();
            let argument =
                if self.check(TokenType::Identifier) && self.peek_nth(1).kind == TokenType::Colon {
                    let name = self.expect_identifier()?;
                    self.expect(TokenType::Colon)?;
                    if !names.insert(name.clone()) {
                        return Err(parser_error(
                            format!("Doodda '{}' laba jeer ayaa la bixiyay.", name),
                            marker.line,
                            marker.col,
                        ));
                    }
                    named = true;
                    let value = self.parse_logical()?;
                    self.locate(
                        Expr::NamedArgument {
                            name,
                            value: Box::new(value),
                        },
                        &marker,
                    )
                } else {
                    if named {
                        return Err(parser_error(
                            "Dood boos leh ma raaci karto dood magac leh.",
                            marker.line,
                            marker.col,
                        ));
                    }
                    self.parse_logical()?
                };
            args.push(argument);
            if self.at_end() {
                break;
            }
            if !self.check(TokenType::RParen) {
                self.expect(TokenType::Comma)?;
            }
        }
        self.expect(TokenType::RParen)?;
        Ok(args)
    }

    // --- Expression precedence (low to high: logical → comparison → additive → multiplicative → unary → postfix → primary) ---

    fn parse_logical(&mut self) -> Result<Expr, SoplangError> {
        let mut left = self.parse_comparison()?;
        loop {
            let marker = self.peek().clone();
            let tok = self.peek();
            let (op, _line, _col) = match &tok.kind {
                TokenType::Or => ("||", tok.line, tok.col),
                TokenType::And => ("&&", tok.line, tok.col),
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = self.locate(
                Expr::BinaryOp {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                &marker,
            );
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, SoplangError> {
        let mut left = self.parse_expression()?;
        loop {
            let marker = self.peek().clone();
            let t = self.peek();
            let op = match &t.kind {
                TokenType::EqEq => {
                    self.advance();
                    "=="
                }
                TokenType::NotEq => {
                    self.advance();
                    "!="
                }
                TokenType::Greater => {
                    self.advance();
                    ">"
                }
                TokenType::Less => {
                    self.advance();
                    "<"
                }
                TokenType::GreaterEq => {
                    self.advance();
                    ">="
                }
                TokenType::LessEq => {
                    self.advance();
                    "<="
                }
                _ => break,
            };
            let right = self.parse_expression()?;
            left = self.locate(
                Expr::BinaryOp {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                &marker,
            );
        }
        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<Expr, SoplangError> {
        let mut left = self.parse_term()?;
        loop {
            let marker = self.peek().clone();
            let t = self.peek();
            let op = match &t.kind {
                TokenType::Plus => {
                    self.advance();
                    "+"
                }
                TokenType::Minus => {
                    self.advance();
                    "-"
                }
                _ => break,
            };
            let right = self.parse_term()?;
            left = self.locate(
                Expr::BinaryOp {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                &marker,
            );
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, SoplangError> {
        let mut left = self.parse_factor()?;
        loop {
            let marker = self.peek().clone();
            let t = self.peek();
            let op = match &t.kind {
                TokenType::Star => {
                    self.advance();
                    "*"
                }
                TokenType::Slash => {
                    self.advance();
                    "/"
                }
                TokenType::Modulo => {
                    self.advance();
                    "%"
                }
                _ => break,
            };
            let right = self.parse_factor()?;
            left = self.locate(
                Expr::BinaryOp {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                &marker,
            );
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, SoplangError> {
        let marker = self.peek().clone();
        let expr = self.parse_factor_inner()?;
        Ok(self.locate(expr, &marker))
    }

    fn parse_factor_inner(&mut self) -> Result<Expr, SoplangError> {
        if self.check(TokenType::Not) {
            self.advance();
            let expr = self.parse_factor()?;
            return Ok(Expr::UnaryOp {
                op: "!".to_string(),
                expr: Box::new(expr),
            });
        }
        if self.check(TokenType::Minus) {
            self.advance();
            let expr = self.parse_factor()?;
            return Ok(Expr::UnaryOp {
                op: "-".to_string(),
                expr: Box::new(expr),
            });
        }
        if self.check(TokenType::Plus) {
            self.advance();
            return self.parse_factor();
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, SoplangError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(TokenType::Dot) {
                self.advance();
                let marker = self.peek().clone();
                let prop = self.expect_member_name()?;
                if self.check(TokenType::LParen) {
                    self.expect(TokenType::LParen)?; // consume '('
                    let args = self.parse_call_args()?;
                    expr = self.locate(
                        Expr::MethodCall {
                            obj: Box::new(expr),
                            method: prop,
                            args,
                        },
                        &marker,
                    );
                } else {
                    expr = self.locate(
                        Expr::Property {
                            obj: Box::new(expr),
                            prop,
                        },
                        &marker,
                    );
                }
            } else if self.check(TokenType::LBracket) {
                let marker = self.peek().clone();
                self.advance();
                let idx = self.parse_logical()?;
                self.expect(TokenType::RBracket)?;
                expr = self.locate(
                    Expr::Index {
                        obj: Box::new(expr),
                        idx: Box::new(idx),
                    },
                    &marker,
                );
            } else if self.check(TokenType::LParen)
                && self.tokens[self.current - 1].line == self.peek().line
            {
                let marker = self.peek().clone();
                self.advance();
                let args = self.parse_call_args()?;
                expr = self.locate(
                    Expr::Invoke {
                        callee: Box::new(expr),
                        args,
                    },
                    &marker,
                );
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, SoplangError> {
        let marker = self.peek().clone();
        let expr = self.parse_primary_inner()?;
        Ok(self.locate(expr, &marker))
    }

    fn parse_primary_inner(&mut self) -> Result<Expr, SoplangError> {
        let line = self.peek().line;
        let col = self.peek().col;

        match &self.peek().kind {
            TokenType::Number => {
                let s = self.peek().lexeme.clone();
                self.advance();
                if s.contains('.') {
                    let x: f64 = s.parse().map_err(|_| {
                        parser_error(format!("Tiro aan sax ahayn: {}", s), line, col)
                    })?;
                    Ok(Expr::Literal(Literal::Float(x)))
                } else {
                    let n: i64 = s.parse().map_err(|_| {
                        parser_error(format!("Tiro aan sax ahayn: {}", s), line, col)
                    })?;
                    Ok(Expr::Literal(Literal::Int(n)))
                }
            }
            TokenType::FormattedString => {
                let token = self.peek().clone();
                self.advance();
                let mut parts = Vec::new();
                for part in token.format_parts {
                    match part {
                        super::token::FormatPart::Text(text) => {
                            parts.push(InterpolationPart::Text(text))
                        }
                        super::token::FormatPart::Field {
                            tokens,
                            format,
                            format_token,
                        } => {
                            let span = format_token
                                .as_ref()
                                .and_then(|token| self.span(token))
                                .or_else(|| tokens.first().and_then(|token| self.span(token)));
                            let mut parser = Parser::new(tokens);
                            parser.source = self.source.clone();
                            let expression = parser.parse_single_expression()?;
                            self.contextual_keywords.extend(parser.contextual_keywords);
                            parts.push(InterpolationPart::Value {
                                expression,
                                format,
                                span,
                            });
                        }
                    }
                }
                Ok(Expr::Interpolated { parts })
            }
            TokenType::String => {
                let lex = self.peek().lexeme.clone();
                self.advance();
                Ok(Expr::Literal(Literal::Str(lex)))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            TokenType::Cusub => {
                self.advance();
                let class_name = self.parse_dotted_name()?;
                self.expect(TokenType::LParen)?;
                let args = self.parse_call_args()?;
                Ok(Expr::New { class_name, args })
            }
            TokenType::Hawl => self.parse_anonymous_function(),
            TokenType::Nafta => {
                self.advance();
                Ok(Expr::Identifier("nafta".to_string()))
            }
            TokenType::Waalid => {
                self.advance();
                Ok(Expr::Identifier("waalid".to_string()))
            }
            TokenType::LParen => {
                self.advance();
                let e = self.parse_logical()?;
                self.expect(TokenType::RParen)?;
                Ok(e)
            }
            TokenType::LBracket => self.parse_list_literal(),
            TokenType::LBrace => self.parse_object_literal(),
            TokenType::Identifier
            | TokenType::Qor
            | TokenType::Gelin
            | TokenType::Abn
            | TokenType::Jajab
            | TokenType::Qoraal
            | TokenType::Bool
            | TokenType::Teed
            | TokenType::Walax => {
                let name = self.current_token_call_name();
                self.advance();
                if self.check(TokenType::LParen) {
                    self.parse_call_rest(name)
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            _ => Err(parser_error(
                format!(
                    "Waxaa la filayay weedh, laakiin waxaa la helay {:?}",
                    self.peek().lexeme
                ),
                line,
                col,
            )),
        }
    }

    fn parse_list_literal(&mut self) -> Result<Expr, SoplangError> {
        self.expect(TokenType::LBracket)?;
        let mut elements = Vec::new();
        while !self.check(TokenType::RBracket) {
            elements.push(self.parse_logical()?);
            if !self.check(TokenType::RBracket) {
                self.expect(TokenType::Comma)?;
            }
        }
        self.expect(TokenType::RBracket)?;
        Ok(Expr::List(elements))
    }

    fn parse_object_literal(&mut self) -> Result<Expr, SoplangError> {
        self.expect(TokenType::LBrace)?;
        let mut pairs = Vec::new();
        while !self.check(TokenType::RBrace) {
            let key = match &self.peek().kind {
                TokenType::Identifier => self.peek().lexeme.clone(),
                TokenType::String => self.peek().lexeme.clone(),
                _ => {
                    let t = self.peek();
                    return Err(parser_error(
                        "Waxaa la filayay magac astaame (magac ama qoraal)",
                        t.line,
                        t.col,
                    ));
                }
            };
            self.advance();
            self.expect(TokenType::Colon)?;
            let value = self.parse_logical()?;
            pairs.push((key, value));
            if !self.check(TokenType::RBrace) {
                self.expect(TokenType::Comma)?;
            }
        }
        self.expect(TokenType::RBrace)?;
        Ok(Expr::Object(pairs))
    }

    fn parse_test(&mut self) -> Result<Stmt, SoplangError> {
        self.advance();
        let name = if self.peek().kind == TokenType::String {
            let name = self.peek().lexeme.clone();
            self.advance();
            name
        } else {
            return Err(parser_error(
                "tijaabo waxay u baahan tahay magac qoraal ah.",
                self.peek().line,
                self.peek().col,
            ));
        };
        let expect_error =
            self.peek().kind == TokenType::Identifier && self.peek().lexeme == "qalad";
        if expect_error {
            self.advance();
        }
        let expected_code = if expect_error && self.peek().kind == TokenType::LParen {
            self.advance();
            if self.peek().kind != TokenType::String || self.peek().lexeme.is_empty() {
                return Err(parser_error(
                    "Summadda khaladku waa inay noqotaa qoraal aan madhnayn.",
                    self.peek().line,
                    self.peek().col,
                ));
            }
            let code = self.peek().lexeme.clone();
            self.advance();
            self.expect(TokenType::RParen)?;
            Some(code)
        } else {
            None
        };
        self.expect(TokenType::LBrace)?;
        let body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        Ok(Stmt::Test {
            name,
            expect_error,
            expected_code,
            body,
        })
    }

    fn parse_fixture(&mut self, setup: bool) -> Result<Stmt, SoplangError> {
        self.advance();
        self.expect(TokenType::LBrace)?;
        let body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        Ok(Stmt::Fixture { setup, body })
    }
}

fn token_to_type_ann(k: &TokenType) -> TypeAnnotation {
    match k {
        TokenType::Null => TypeAnnotation::Null,
        TokenType::Abn => TypeAnnotation::Abn,
        TokenType::Jajab => TypeAnnotation::Jajab,
        TokenType::Qoraal => TypeAnnotation::Qoraal,
        TokenType::Bool => TypeAnnotation::Bool,
        TokenType::Teed => TypeAnnotation::Teed,
        TokenType::Walax => TypeAnnotation::Walax,
        _ => TypeAnnotation::Dynamic,
    }
}

fn token_type_to_name(k: &TokenType) -> String {
    match k {
        TokenType::Qor => "qor".to_string(),
        TokenType::Gelin => "gelin".to_string(),
        TokenType::Abn => "abn".to_string(),
        TokenType::Jajab => "jajab".to_string(),
        TokenType::Qoraal => "qoraal".to_string(),
        TokenType::Bool => "bool".to_string(),
        TokenType::Teed => "teed".to_string(),
        TokenType::Walax => "walax".to_string(),
        _ => "?".to_string(),
    }
}

fn token_name_expected(k: &TokenType) -> &'static str {
    match k {
        TokenType::LParen => "'('",
        TokenType::RParen => "')'",
        TokenType::LBrace => "'{'",
        TokenType::RBrace => "'}'",
        TokenType::LBracket => "'['",
        TokenType::RBracket => "']'",
        TokenType::Comma => "','",
        TokenType::Colon => "':'",
        TokenType::Semicolon => "';'",
        TokenType::Assign => "'='",
        TokenType::Identifier => "magac",
        TokenType::String | TokenType::FormattedString => "qoraal",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::frontend::ast::{Expr, Literal, Stmt};
    use crate::frontend::lexer::Lexer;

    #[test]
    fn parse_hello() {
        let source = r#"qor("Salaan, Adduunka!")"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Stmt::Expr(Expr::Call { name, args }) => {
                assert_eq!(name, "qor");
                assert_eq!(args.len(), 1);
                assert!(
                    matches!(&args[0], Expr::Literal(Literal::Str(s)) if s == "Salaan, Adduunka!")
                );
            }
            _ => panic!("expected qor(...) statement"),
        }
    }

    #[test]
    fn parse_var_and_if() {
        let source = r#"
            door x = 10
            haddii (x > 5) {
                qor("run")
            }
        "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse().unwrap();
        assert!(stmts.len() >= 2);
    }
}
