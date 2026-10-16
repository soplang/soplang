//! Recursive descent parser: tokens → AST (Vec<Stmt>).
//! Parser for Soplang. IMPLEMENTATION_PLAN Phase 2.

use super::ast::{Expr, Literal, Param, Stmt, TypeAnnotation};
use super::token::{Token, TokenType};
use crate::error::{parser_error, SoplangError};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
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
        matches!(self.peek().kind, TokenType::Identifier)
            && matches!(self.peek_nth(1).kind, TokenType::Identifier)
            && matches!(self.peek_nth(2).kind, TokenType::Assign)
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

        match &t.kind {
            TokenType::Haddii => return self.parse_if(line, col),
            TokenType::Dooro => return self.parse_switch(line, col),
            TokenType::Door => return self.parse_var_decl(false, false, line, col),
            TokenType::Madoor => return self.parse_madoor(line, col),
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
            TokenType::Qaab => return self.parse_class_def(),
            TokenType::LBrace => return self.parse_block(),
            TokenType::Identifier if self.looks_like_class_decl() => {
                let class_name = self.peek().lexeme.clone();
                self.advance();
                return self.parse_var_decl_rest(
                    TypeAnnotation::Class(class_name),
                    false,
                    line,
                    col,
                );
            }
            TokenType::Identifier | TokenType::Nafta | TokenType::Waalid => {
                return self.parse_identifier_stmt(line, col)
            }
            _ => {}
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
        let (type_ann, is_static) = if matches!(
            self.peek().kind,
            TokenType::Abn
                | TokenType::Jajab
                | TokenType::Qoraal
                | TokenType::Bool
                | TokenType::Teed
                | TokenType::Walax
        ) {
            (token_to_type_ann(&self.peek().kind), true)
        } else if self.looks_like_class_decl() {
            let class_name = self.peek().lexeme.clone();
            self.advance();
            return self.parse_var_decl_rest(TypeAnnotation::Class(class_name), true, line, col);
        } else {
            (TypeAnnotation::Dynamic, false)
        };
        if is_static {
            self.advance();
        }
        self.parse_var_decl_rest(type_ann, true, line, col)
    }

    fn parse_var_decl_static(&mut self, line: usize, col: usize) -> Result<Stmt, SoplangError> {
        let type_ann = token_to_type_ann(&self.peek().kind);
        self.advance();
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
        if self.check(TokenType::Kuceli) {
            self.advance();
            return Ok("kuceli".to_string());
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

    fn parse_function_tail(
        &mut self,
    ) -> Result<(Vec<Param>, TypeAnnotation, Vec<Stmt>), SoplangError> {
        self.expect(TokenType::LParen)?;
        let mut params = Vec::new();
        while !self.check(TokenType::RParen) {
            let type_ann = if matches!(
                self.peek().kind,
                TokenType::Abn
                    | TokenType::Jajab
                    | TokenType::Qoraal
                    | TokenType::Bool
                    | TokenType::Teed
                    | TokenType::Walax
            ) {
                let ann = token_to_type_ann(&self.peek().kind);
                self.advance();
                ann
            } else if matches!(self.peek().kind, TokenType::Identifier)
                && matches!(self.peek_nth(1).kind, TokenType::Identifier)
            {
                let class_name = self.peek().lexeme.clone();
                self.advance();
                TypeAnnotation::Class(class_name)
            } else {
                TypeAnnotation::Dynamic
            };
            params.push(Param {
                name: self.expect_identifier()?,
                type_ann,
            });
            if !self.check(TokenType::RParen) {
                self.expect(TokenType::Comma)?;
            }
        }
        self.expect(TokenType::RParen)?;
        let return_ann = if self.check(TokenType::Colon) {
            self.advance();
            if matches!(
                self.peek().kind,
                TokenType::Abn
                    | TokenType::Jajab
                    | TokenType::Qoraal
                    | TokenType::Bool
                    | TokenType::Teed
                    | TokenType::Walax
            ) {
                let ann = token_to_type_ann(&self.peek().kind);
                self.advance();
                ann
            } else if matches!(self.peek().kind, TokenType::Identifier) {
                let class_name = self.peek().lexeme.clone();
                self.advance();
                TypeAnnotation::Class(class_name)
            } else {
                return Err(parser_error(
                    "Waxaa la filayay nooca celi (abn/jajab/qoraal/bool/teed/walax)",
                    self.peek().line,
                    self.peek().col,
                ));
            }
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
        let name = self.peek().lexeme.clone();
        self.advance(); // gelin or similar
        let call = self.parse_call_rest(name)?;
        Ok(Stmt::Expr(call))
    }

    fn parse_if(&mut self, _line: usize, _col: usize) -> Result<Stmt, SoplangError> {
        self.advance(); // haddii
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
            self.advance();
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
            self.advance();
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
        self.advance(); // dooro
        self.expect(TokenType::LParen)?;
        let expr = self.parse_logical()?;
        self.expect(TokenType::RParen)?;
        self.expect(TokenType::LBrace)?;

        let mut cases = Vec::new();
        let mut default = None;

        while !self.check(TokenType::RBrace) {
            if self.check(TokenType::Xaalad) {
                self.advance();
                let case_val = self.parse_logical()?;
                self.expect(TokenType::LBrace)?;
                let body = self.parse_block_stmts()?;
                self.expect(TokenType::RBrace)?;
                cases.push((case_val, body));
            } else if self.check(TokenType::Ugudambeyn)
                || (self.looks_like_kale_branch() && self.peek_nth(1).kind == TokenType::LBrace)
            {
                self.advance();
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
        self.advance(); // kuceli
        self.expect(TokenType::LParen)?;
        let var = self.expect_identifier()?;
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
        self.advance();
        let end = self.parse_expression()?;
        let step = if self.check(TokenType::Colon) {
            self.advance();
            if self.check(TokenType::Colon) {
                self.advance();
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
        self.advance(); // tijaabi or fasax
        self.expect(TokenType::LBrace)?;
        let try_body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        self.expect(TokenType::Qabo)?;
        self.expect(TokenType::LParen)?;
        let err_var = self.expect_identifier()?;
        self.expect(TokenType::RParen)?;
        self.expect(TokenType::LBrace)?;
        let catch_body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        Ok(Stmt::TryCatch {
            try_body,
            err_var,
            catch_body,
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
                    "Waxaa la filayay qoraal (file path)",
                    t.line,
                    t.col,
                ));
            }
        };
        Ok(Stmt::Import(path))
    }

    fn parse_class_def(&mut self) -> Result<Stmt, SoplangError> {
        let t = self.peek();
        let (line, col) = (t.line, t.col);
        self.advance(); // qaab
        let name = self.expect_identifier()?;
        let parent = if self.check(TokenType::Dhaxal) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        self.expect(TokenType::LBrace)?;
        let body = self.parse_block_stmts()?;
        self.expect(TokenType::RBrace)?;
        for s in &body {
            if !matches!(s, Stmt::FuncDef { .. }) {
                return Err(parser_error(
                    "Jirka qaabka waa in uu ka koobnaado hawl (methods) kaliya",
                    line,
                    col,
                ));
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
        let name = match &self.peek().kind {
            TokenType::Nafta => "nafta".to_string(),
            TokenType::Waalid => "waalid".to_string(),
            _ => self.peek().lexeme.clone(),
        };
        self.advance();

        // Chain of .prop and [index] - then maybe = expr
        let mut left = Expr::Identifier(name);

        while self.check(TokenType::Dot) || self.check(TokenType::LBracket) {
            if self.check(TokenType::Dot) {
                self.advance();
                let prop = self.expect_member_name()?;
                if self.check(TokenType::LParen) {
                    self.expect(TokenType::LParen)?; // consume '('
                    let args = self.parse_call_args()?;
                    left = Expr::MethodCall {
                        obj: Box::new(left),
                        method: prop,
                        args,
                    };
                } else {
                    left = Expr::Property {
                        obj: Box::new(left),
                        prop,
                    };
                }
            } else {
                self.advance(); // [
                let idx = self.parse_logical()?;
                self.expect(TokenType::RBracket)?;
                left = Expr::Index {
                    obj: Box::new(left),
                    idx: Box::new(idx),
                };
            }
        }

        if self.check(TokenType::Assign) {
            self.advance();
            let value = self.parse_logical()?;
            return Ok(Stmt::Assign {
                target: left,
                value,
                line,
                col,
            });
        }

        if self.check(TokenType::LParen) {
            self.expect(TokenType::LParen)?; // consume '('
            let args = self.parse_call_args()?;
            let call = Expr::Call {
                name: match &left {
                    Expr::Identifier(n) => n.clone(),
                    _ => return Err(parser_error("Waxaa la filayay magac hawl", line, col)),
                },
                args,
            };
            return Ok(Stmt::Expr(call));
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
        while !self.check(TokenType::RParen) {
            args.push(self.parse_logical()?);
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
            let tok = self.peek();
            let (op, _line, _col) = match &tok.kind {
                TokenType::Or => ("||", tok.line, tok.col),
                TokenType::And => ("&&", tok.line, tok.col),
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp {
                op: op.to_string(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, SoplangError> {
        let mut left = self.parse_expression()?;
        loop {
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
            left = Expr::BinaryOp {
                op: op.to_string(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<Expr, SoplangError> {
        let mut left = self.parse_term()?;
        loop {
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
            left = Expr::BinaryOp {
                op: op.to_string(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, SoplangError> {
        let mut left = self.parse_factor()?;
        loop {
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
            left = Expr::BinaryOp {
                op: op.to_string(),
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr, SoplangError> {
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
                let prop = self.expect_member_name()?;
                if self.check(TokenType::LParen) {
                    self.expect(TokenType::LParen)?; // consume '('
                    let args = self.parse_call_args()?;
                    expr = Expr::MethodCall {
                        obj: Box::new(expr),
                        method: prop,
                        args,
                    };
                } else {
                    expr = Expr::Property {
                        obj: Box::new(expr),
                        prop,
                    };
                }
            } else if self.check(TokenType::LBracket) {
                self.advance();
                let idx = self.parse_logical()?;
                self.expect(TokenType::RBracket)?;
                expr = Expr::Index {
                    obj: Box::new(expr),
                    idx: Box::new(idx),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, SoplangError> {
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
                let class_name = self.expect_identifier()?;
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
                    "Waxaa la filayay expression, laakiin waxaa la helay {:?}",
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
                        "Waxaa la filayay magac astaame (identifier ama qoraal)",
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
}

fn token_to_type_ann(k: &TokenType) -> TypeAnnotation {
    match k {
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
        TokenType::String => "qoraal",
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
