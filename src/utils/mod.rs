pub mod error;
pub mod function;

#[inline]
pub fn valid_symbol_chr(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric()
}

pub fn is_one_word(s: &str) -> bool {
    s.chars().all(valid_symbol_chr)
}
