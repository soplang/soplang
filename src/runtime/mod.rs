//! Runtime: values, stdlib builtins, and C ABI for backends.

pub mod abi;
mod async_files;
mod async_http;
mod async_net;
mod base64;
mod binary_files;
mod bytes;
mod channels;
mod context;
mod contracts;
mod csv;
pub(crate) mod debugger;
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
mod multipart;
mod net;
mod numeric;
mod object_ops;
mod patterns;
mod process;
mod report;
mod roots;
mod sets;
pub mod stdlib;
mod tasks;
mod template;
mod text;
mod text_files;
mod time;
mod tuples;
mod unicode_case;
mod url;
pub mod value;
mod web_support;
mod websocket;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub mod windows_abi;

pub use abi::*;
pub(crate) use context::with_databases;
pub use context::RuntimeContext;
pub use debugger::{parse_breakpoint, DebugConfig};
pub(crate) use filesystem::replace_bytes as replace_file_bytes;
pub(crate) use filesystem::replace_bytes_with_permissions as replace_file_with_permissions;
pub use gc::{CollectionReport, MemoryStats};
pub use iterators::IteratorData;
pub use process::set_program_arguments;
pub use report::{
    clear_coverage, coverage_lines, profile_counts, record_coverage, set_profile, CoverageMap,
    CoverageReport, ExecutionReport, FunctionProfile,
};
pub use roots::RootedValue;
pub use sets::SetData;
pub use tuples::TupleData;
pub use value::{value_to_string, NativeFn, Value};
