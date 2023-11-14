use phf::phf_map;

// Represents the various operators (punctuation)
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum OpName {
    Add,
    Sub,
    Div,
    Mul,
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    TmpAssign,
    Define,
    Pop,
    Swap,
    Dup,
    Call,
}

// Enum that represents the various types of tokens the `Lexer` can consume and return
#[derive(Clone, Debug)]
pub enum Token {
    Word(String),
    Number(i32),
    Register(String),
    String(String),
    Variable(String),
    Operator(OpName),
    Newline,
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
}

use OpName::*;
// Static map representing the different punctuation's mapping to `OpName`
pub static OP_MAP: phf::Map<&'static str, OpName> = phf_map! {
    "+" => Add,
    "-" => Sub,
    "/" => Div,
    "*" => Mul,
    "=" => Eq,
    "!=" => Neq,
    ">" => Gt,
    "<" => Lt,
    ">=" => Gte,
    "<=" => Lte,
    "<-" => Define,
    "!" => TmpAssign,
    ";" => Pop,
    ":" => Swap,
    "." => Dup,
    "?" => Call,
};
