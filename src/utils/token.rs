use phf::phf_map;

#[derive(Clone, Copy, Debug)]
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
