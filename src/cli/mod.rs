//! CLI: REPL and interactive shell.

mod data;
mod debug;
mod document;
pub mod lsp;
pub mod launcher;
mod options;
pub mod project;
pub mod session;
pub mod shell;
mod testing;
pub mod tools;

pub use session::ReplSession;
pub use shell::Shell;
