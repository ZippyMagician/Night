use std::fmt::{self, Display};
use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::utils::error::{night_err, Status};

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
    #[inline]
    pub fn is_num(&self) -> bool {
        match self.t {
            Type::Num(_) => true,
            _ => false,
        }
    }

    pub fn as_num(self) -> Status<i32> {
        match self.t {
            Type::Num(n) => Ok(n),
            _ => night_err!(NaN),
        }
    }

    #[inline]
    pub fn is_str(&self) -> bool {
        match self.t {
            Type::Str(_) => true,
            _ => false,
        }
    }

    pub fn as_str(self) -> Status<String> {
        match self.t {
            Type::Str(s) => Ok(s),
            _ => night_err!(UnsupportedType, "Expected string."),
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

                    night_err!(UnsupportedType, concat!("Cannot call '", $lit, "' on non-numbers."))
                }
            }
        )*
    }
}

impl_arith_ops! {
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
            Type::Str(s) => write!(f, "\"{s}\""),
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

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self {
            t: Type::Str(value.to_string()),
        }
    }
}
