mod defs;

pub use defs::OP_MAP;

// Represents the various operators (punctuation)
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum Operator {
    // `+`
    Add,
    // `-`
    Sub,
    // `/`
    Div,
    // `*`
    Mul,
    // `=`
    Eq,
    // `!=`
    Neq,
    // `>`
    Gt,
    // `>=`
    Gte,
    // `<`
    Lt,
    // `<=`
    Lte,
    // `!`
    Assign,
    // `;`
    Pop,
    // `:`
    Swap,
    // `.`
    Dup,
    // `?`
    Call,
}

// TODO: impl Operator to write the functions for each
