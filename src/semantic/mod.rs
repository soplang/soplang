//! Semantic analysis: name resolution, types, symbol table.

pub mod analyze;
pub mod scope;

pub use analyze::{
    analyze, analyze_into, analyze_with_options, canonical_class_name, mangle_method, resolve_name,
    resolve_parent_method, AnalyzeOptions, ClassMeta, FunctionMeta, Scope, SymbolTable, VarInfo,
};
pub use scope::Env;
