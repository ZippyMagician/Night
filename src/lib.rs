pub mod lexer;
pub mod operator;
pub mod scope;
pub mod utils;

/// Current program version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
