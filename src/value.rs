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
    pub fn types_match(left: &Self, right: &Self) -> bool {
        match &left.t {
            Type::Num(_) => matches!(right.t, Type::Num(_)),
            Type::Str(_) => matches!(right.t, Type::Str(_)),
        }
    }

    #[inline]
    pub fn is_num(&self) -> bool {
        match self.t {
            Type::Num(_) => true,
            _ => false,
        }
    }

    #[inline]
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

    #[inline]
    pub fn as_str(self) -> Status<String> {
        match self.t {
            Type::Str(s) => Ok(s),
            _ => night_err!(UnsupportedType, "Expected string."),
        }
    }

    #[inline]
    pub fn as_bool(self) -> Status<bool> {
        match self.t {
            Type::Num(n) if n == 0 => Ok(false),
            Type::Num(n) if n > 0 => Ok(true),
            Type::Num(_) => night_err!(
                UnsupportedType,
                "To coerce an integer into a boolean, it must be positive."
            ),
            _ => night_err!(NaN),
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match &self.t {
            Type::Num(left) => match &other.t {
                Type::Num(right) => left == right,
                _ => false,
            },
            Type::Str(left) => match &other.t {
                Type::Str(right) => left == right,
                _ => false,
            },
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match &self.t {
            Type::Num(left) => match other.t {
                Type::Num(right) => left.partial_cmp(&right),
                _ => None,
            },
            Type::Str(left) => match &other.t {
                Type::Str(right) => left.partial_cmp(&right),
                _ => None,
            },
        }
    }
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

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self {
            t: Type::Num(if value { 1 } else { 0 }),
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
