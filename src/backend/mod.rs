//! Compiler backends: Cranelift JIT and LLVM AOT.

pub mod cranelift;
pub mod llvm;
pub mod llvm_ir;
#[cfg(feature = "embedded-llvm")]
pub mod llvm_native;
