//! Soplang library for use by the binary and integration tests.

pub mod analysis;
pub mod backend;
pub mod cli;
pub mod data;
pub mod error;
pub mod frontend;
pub mod hir;
pub mod package;
pub mod runtime;
pub mod semantic;

pub use cli::ReplSession;
pub use error::{format_error_with_source, SoplangError};
pub use frontend::{
    ast, lexer, parser, token, Expr, Lexer, Literal, Param, Parser, Stmt, Token, TokenType,
    TypeAnnotation,
};
pub use hir::{BinOpKind, HirFunction, HirInstr, HirLowering, HirModule, UnOpKind};
pub use runtime::set_program_arguments;
pub use runtime::Value;
pub use semantic::{
    analyze, analyze_with_options, AnalyzeOptions, ClassMeta, FunctionMeta, Scope, SymbolTable,
    VarInfo,
};

use std::path::Path;

/// Run source code through the compiled pipeline (semantic → HIR → Cranelift JIT).
pub fn run_source(
    source: &str,
    path: Option<&Path>,
    print_ast: bool,
    dump_hir: bool,
    strict: bool,
) -> Result<(), SoplangError> {
    if let Some(path) = path {
        package::load_beside_source(path)?;
    }
    let parsed = if path.is_none() {
        frontend::parse_snippet(source)?
    } else {
        frontend::parse_source(source, path)?
    };
    let stmts = frontend::imports::resolve_imports(parsed, path)?;
    if print_ast {
        for s in &stmts {
            print!("{}", s);
        }
        return Ok(());
    }
    let sym = semantic::analyze_with_options(
        &stmts,
        semantic::AnalyzeOptions {
            strict,
            repl: false,
        },
    )?;
    let hir = HirLowering::lower(&sym, &stmts);
    if dump_hir {
        for f in &hir.functions {
            println!("\n--- {} ---", f.name);
            for instr in &f.body {
                println!("{}", instr);
            }
        }
        println!("\n--- qaybta guud ---");
        for instr in &hir.top_level {
            println!("{}", instr);
        }
        return Ok(());
    }
    let mut backend = backend::cranelift::CraneliftBackend::new()?;
    backend.compile_module(&hir, &sym)?;
    backend.run_main()
}

/// Run statements that were already parsed and import-resolved.
pub fn run_statements(stmts: &[frontend::ast::Stmt], strict: bool) -> Result<(), SoplangError> {
    let sym = semantic::analyze_with_options(
        stmts,
        semantic::AnalyzeOptions {
            strict,
            repl: false,
        },
    )?;
    let hir = HirLowering::lower(&sym, stmts);
    let mut backend = backend::cranelift::CraneliftBackend::new()?;
    backend.compile_module(&hir, &sym)?;
    backend.run_main()
}

/// For REPL and -c: if source is a single expression (not already qor(...)), wrap in qor(...) so the value is printed.
pub fn maybe_wrap_for_repl(source: &str) -> String {
    let tokens = match Lexer::new(source).tokenize() {
        Ok(t) => t,
        Err(_) => return source.to_string(),
    };
    let mut parser = Parser::new(tokens);
    let expr = match parser.parse_single_expression() {
        Ok(e) => e,
        Err(_) => return source.to_string(),
    };
    match &expr {
        Expr::Call { name, .. } if name == "qor" => source.to_string(),
        _ => format!("qor({})", source.trim()),
    }
}

/// Build a standalone executable (AOT path).
///
/// `source_path` is the file being built. `keen` imports are resolved before
/// lowering so the binary does not need those files beside it.
pub fn build_source(
    source: &str,
    source_path: Option<&Path>,
    out_path: &Path,
    opt_level: u8,
    strict: bool,
) -> Result<(), SoplangError> {
    if let Some(path) = source_path {
        package::load_beside_source(path)?;
    }
    let parsed = frontend::parse_source(source, source_path)?;
    let stmts = frontend::imports::resolve_imports(parsed, source_path)?;
    let sym = semantic::analyze_with_options(
        &stmts,
        semantic::AnalyzeOptions {
            strict,
            repl: false,
        },
    )?;
    let hir = HirLowering::lower(&sym, &stmts);
    let backend = backend::llvm::LlvmBackend::new();
    backend.build_executable(&hir, &sym, out_path, opt_level)
}
