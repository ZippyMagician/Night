mod defs;

pub use defs::OP_MAP;
pub use defs::BUILTIN_MAP;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum Operator {
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
    // TODO: more
}
