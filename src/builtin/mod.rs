mod defs;

pub use defs::BUILTIN_MAP;
pub use defs::OP_MAP;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Operator {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Assign,
    Pop,
    Swap,
    Dup,
    Call,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum Builtin {
    Print,
    Inc,
    Dec,
    Def,
    Undef,
    // TODO: more
}
