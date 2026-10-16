//! Runtime: values, stdlib builtins, and C ABI for backends.

pub mod abi;
pub mod stdlib;
pub mod value;

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub mod windows_abi;

pub use abi::*;
pub use value::{value_to_string, NativeFn, Value};
