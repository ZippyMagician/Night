use crate::{interpreter::Night, lexer::Lexer};

pub mod error;
pub mod function;

#[inline]
pub fn valid_symbol_chr(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric()
}

pub fn is_one_word(s: &str) -> bool {
    s.chars().all(valid_symbol_chr)
}

pub fn define_fn(night: &mut Night, def: &'static str) {
    let mut lexer = Lexer::new(def);
    let tokens = lexer.tokenize();
    night.inject_code(tokens);
}
