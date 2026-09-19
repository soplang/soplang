//! Semantic analysis: name resolution, types, symbol table.

pub mod analyze;
pub mod scope;

pub use analyze::{
    analyze, analyze_into, analyze_with_options, mangle_method, resolve_parent_method,
    AnalyzeOptions, ClassMeta, FunctionMeta, resolve_name, Scope, SymbolTable, VarInfo,
};
pub use scope::Env;
