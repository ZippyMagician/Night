mod defs;

pub use defs::BUILTIN_MAP;
pub use defs::OP_MAP;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Operator {
    /// 1 2 + -- 3
    Add,
    /// 3 1 - -- 2
    Sub,
    /// 4 2 / -- 2
    Div,
    /// 3 5 * -- 15
    Mul,
    /// 4 2 % -- 0
    Mod,
    /// 3 2 = -- 0
    Eq,
    /// 3 2 != -- 1
    NotEq,
    /// 2 2 > -- 0
    Greater,
    /// 2 2 >= -- 1
    GreaterEq,
    /// 1 2 < -- 1
    Less,
    /// 1 2 <= -- 1
    LessEq,
    /// 1 ~ -- 0
    Not,
    /// 6 :tmp ! -- 6, [$tmp = 6]
    Assign,
    /// 5 4 6 ; -- 5 4
    Pop,
    /// 3 2 : -- 2 3
    Swap,
    /// 5 . -- 5 5
    Dup,
    /// 4 { 1 + } ? -- 5
    Call,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Builtin {
    /// Pop top value, print to stdout
    Print,
    /// Increment top value by 1
    Inc,
    /// Decrement top value by 1
    Dec,
    /// Symbol definition
    Def,
    /// Symbol undefinition
    Undef,
    /// (temp) symbol / register undefinition
    UndefReg,
    /// n <function> loop -> call function n times
    Loop,
    /// cond <function> if -> call function if cond is truthy
    If,
    /// over ( a b -- a b a )
    Over,
    /// rot ( a b c -- b c a )
    Rot,
    /// rotr ( a b c -- c a b )
    RotRight,
    /// Logical or of two values
    LogicalOr,
    /// Logical and of two values
    LogicalAnd,
    /// Floor a number into an i32
    Floor,
    /// Ceil a number into an i32
    Ceil,
    /// Cast a number to an i32
    CastToInt,
    /// Cast a number to a f32
    CastToFloat,
    /// bind ( op fn -- { op fn } )
    Bind,
}
