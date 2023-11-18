pub mod lexer;
pub mod operator;
pub mod utils;
pub mod scope;

/// Current program version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
