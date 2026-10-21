//! CLI: REPL and interactive shell.

pub mod lsp;
pub mod project;
pub mod session;
pub mod shell;
pub mod tools;

pub use session::ReplSession;
pub use shell::Shell;
