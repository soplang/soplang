//! Runtime: values, stdlib builtins, and C ABI for backends.

pub mod abi;
mod bytes;
mod context;
mod contracts;
mod handles;
mod gc;
mod formatting;
mod iterators;
mod roots;
mod tuples;
mod text;
mod unicode_case;
mod sets;
mod list_ops;
mod object_ops;
mod numeric;
mod text_files;
pub mod stdlib;
pub mod value;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub mod windows_abi;

pub use abi::*;
pub use context::RuntimeContext;
pub use roots::RootedValue;
pub use gc::{CollectionReport, MemoryStats};
pub use iterators::IteratorData;
pub use tuples::TupleData;
pub use sets::SetData;
pub use value::{value_to_string, NativeFn, Value};
