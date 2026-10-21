//! Runtime: values, stdlib builtins, and C ABI for backends.

pub mod abi;
mod base64;
mod binary_files;
mod bytes;
mod context;
mod contracts;
mod csv;
mod file_handles;
mod filesystem;
mod formatting;
mod gc;
mod handles;
mod http;
mod http_server;
mod iterators;
mod json;
mod list_ops;
mod net;
mod numeric;
mod object_ops;
mod patterns;
mod process;
mod roots;
mod sets;
pub mod stdlib;
mod text;
mod text_files;
mod time;
mod tuples;
mod unicode_case;
mod url;
pub mod value;
mod websocket;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub mod windows_abi;

pub use abi::*;
pub use context::RuntimeContext;
pub use gc::{CollectionReport, MemoryStats};
pub use iterators::IteratorData;
pub use process::set_program_arguments;
pub use roots::RootedValue;
pub use sets::SetData;
pub use tuples::TupleData;
pub use value::{value_to_string, NativeFn, Value};
