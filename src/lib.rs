pub mod interpreter;
pub mod lexer;
pub mod builtin;
pub mod scope;
pub mod utils;
pub mod value;

/// Current program version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
