pub mod error;
pub mod function;

// Helper methods here
pub fn is_one_word(s: &str) -> bool {
    s.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
}