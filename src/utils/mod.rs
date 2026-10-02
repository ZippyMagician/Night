use crate::interpreter::{Instr, Night};
use crate::lexer::Lexer;

pub mod error;
pub mod function;

#[inline]
pub fn valid_symbol_chr(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric()
}

pub fn is_one_word(s: &str) -> bool {
    s.chars().all(valid_symbol_chr)
}

pub fn span_instrs(i: Instr, span: usize) -> Instr {
    match i {
        Instr::Push(v, s) if s == usize::MAX => Instr::Push(v, span),
        Instr::PushFunc(f, s) if s == usize::MAX => Instr::PushFunc(f, span),
        _ => i,
    }
}

pub fn define_fns(night: &mut Night, def: &'static str, file: &'static str) {
    let lexer = Lexer::new(def, file);
    let tokens = lexer.tokenize();
    night.push_new_code(def, tokens);
    night.exec();
}
