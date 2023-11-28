use std::fmt::{self, Display};
use std::ops::{Add, Sub, Mul, Div, Rem};

use crate::utils::error::{NightError, Status};

#[derive(Clone, Debug)]
enum Type {
    Num(i32),
    Str(String),
}

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Value {
    t: Type,
}

impl Value {
    pub fn is_num(&self) -> bool {
        match self.t {
            Type::Num(_) => true,
            _ => false,
        }
    }

    pub fn is_str(&self) -> bool {
        match self.t {
            Type::Str(_) => true,
            _ => false,
        }
    }
}

// Macro to quickly impl the various arithmetic operations for `Value`
macro_rules! impl_arith_ops {
    ($($name:ident, $f:ident, $lit:literal, [$a1:ident, $a2:ident] $operation:block);*;) => {
        $(
            impl $name for Value {
                type Output = Status<Value>;

                fn $f(self, rhs: Self) -> Self::Output {
                    if let Type::Num($a1) = self.t {
                        if let Type::Num($a2) = rhs.t {
                            return Ok(Value::from($operation));
                        }
                    }

                    Err(NightError::UnsupportedType(
                        concat!("Cannot call '", $lit, "' on non-numbers.").to_string(),
                    ))
                }
            }
        )*
    }
}

impl_arith_ops!{
    Add, add, "add", [l, r] {l + r};
    Sub, sub, "sub", [l, r] {l - r};
    Mul, mul, "mul", [l, r] {l * r};
    Div, div, "div", [l, r] {l / r};
    Rem, rem, "mod", [l, r] {l % r};
}



impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.t {
            Type::Num(l) => write!(f, "{l}"),
            Type::Str(s) => write!(f, "{s}"),
        }
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self {
            t: Type::Num(value),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self {
            t: Type::Str(value),
        }
    }
}
