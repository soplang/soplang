//! Runtime: values, stdlib builtins, and C ABI for backends.

pub mod abi;
mod bytes;
mod context;
mod handles;
mod gc;
mod formatting;
mod iterators;
mod roots;
pub mod stdlib;
pub mod value;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub mod windows_abi;

pub use abi::*;
pub use context::RuntimeContext;
pub use roots::RootedValue;
pub use gc::{CollectionReport, MemoryStats};
pub use iterators::IteratorData;
pub use value::{value_to_string, NativeFn, Value};
