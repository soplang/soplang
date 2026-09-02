//! One JIT session for the REPL. Functions, classes, and globals survive across lines.

use std::collections::HashMap;
use std::path::Path;

use crate::backend::cranelift::CraneliftBackend;
use crate::error::SoplangError;
use crate::frontend::{Lexer, Parser};
use crate::hir::HirLowering;
use crate::runtime;
use crate::semantic::{self, AnalyzeOptions, ClassMeta, Scope, SymbolTable, VarInfo};

pub struct ReplSession {
    backend: CraneliftBackend,
    globals: HashMap<String, VarInfo>,
    classes: HashMap<String, ClassMeta>,
    /// Compiled snippets. Name pointers baked into JIT code point into these modules.
    hirs: Vec<Box<crate::hir::HirModule>>,
    strict: bool,
}

impl ReplSession {
    pub fn new(strict: bool) -> Result<Self, SoplangError> {
        Ok(Self {
            backend: CraneliftBackend::new()?,
            globals: HashMap::new(),
            classes: HashMap::new(),
            hirs: Vec::new(),
            strict,
        })
    }

    /// Compile and run one snippet. `path` is set when the snippet comes from a file.
    pub fn eval(&mut self, source: &str, path: Option<&Path>) -> Result<(), SoplangError> {
        runtime::reset_try_state();
        let tokens = Lexer::new(source).tokenize()?;
        let stmts = crate::frontend::imports::resolve_imports(Parser::new(tokens).parse()?, path)?;
        let mut sym = self.prelude();
        semantic::analyze_into(
            &mut sym,
            &stmts,
            AnalyzeOptions {
                strict: self.strict,
                repl: true,
            },
        )?;
        let hir = HirLowering::lower(&sym, &stmts);
        let boxed = Box::new(hir);
        let compiled = self.backend.compile_incremental(&boxed, &sym);
        self.hirs.push(boxed);
        compiled?;
        self.backend.run_main()?;
        self.remember(&sym);
        Ok(())
    }

    fn prelude(&self) -> SymbolTable {
        let mut sym = SymbolTable::default();
        let mut scope = Scope::default();
        for (name, info) in &self.globals {
            scope.vars.insert(name.clone(), info.clone());
        }
        sym.session_globals = self.globals.keys().cloned().collect();
        sym.scopes.push(scope);
        sym.classes = self.classes.clone();
        sym
    }

    fn remember(&mut self, sym: &SymbolTable) {
        if let Some(scope) = sym.scopes.first() {
            self.globals = scope.vars.clone();
        }
        self.classes = sym.classes.clone();
    }
}
