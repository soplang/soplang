//! Phase 5 AOT backend entrypoint.

use std::path::Path;

use crate::error::SoplangError;
use crate::hir::HirModule;
use crate::semantic::SymbolTable;

use super::llvm_ir::{emit_llvm_ir, link_llvm_binary_with_opt};

pub struct LlvmBackend;

impl LlvmBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn build_executable(
        &self,
        hir: &HirModule,
        sym: &SymbolTable,
        out_path: &Path,
        opt_level: u8,
    ) -> Result<(), SoplangError> {
        let ir = emit_llvm_ir(hir, sym)?;
        link_llvm_binary_with_opt(&ir, out_path, opt_level)
    }
}
