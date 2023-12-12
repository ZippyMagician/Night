use std::fmt::{self, Display};
use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::utils::error::{night_err, Status};

#[derive(Clone, Debug)]
enum Type {
    Int(i32),
    Float(f32),
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
            Type::Int(_) | Type::Float(_) => matches!(right.t, Type::Int(_) | Type::Float(_)),
            Type::Str(_) => matches!(right.t, Type::Str(_)),
        }
    }

    #[inline]
    pub fn is_num(&self) -> bool {
        match self.t {
            Type::Int(_) | Type::Float(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        match self.t {
            Type::Int(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        match self.t {
            Type::Float(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn as_int(self) -> Status<i32> {
        match self.t {
            Type::Int(n) => Ok(n),
            Type::Float(n) => Ok(n as i32),
            _ => night_err!(NaN),
        }
    }

    #[inline]
    pub fn as_float(self) -> Status<f32> {
        match self.t {
            Type::Int(n) => Ok(n as f32),
            Type::Float(n) => Ok(n),
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
            Type::Int(n) if n == 0 => Ok(false),
            Type::Int(n) if n > 0 => Ok(true),
            Type::Int(_) => night_err!(
                UnsupportedType,
                "To coerce an integer into a boolean, it must be positive."
            ),
            Type::Float(n) if n == 0. => Ok(false),
            Type::Float(n) if n > 0. => Ok(true),
            Type::Float(_) => night_err!(
                UnsupportedType,
                "To coerce a float into a boolean, it must be positive."
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
                    if self.is_float() || rhs.is_float() {
                        let $a1 = self.as_float()?;
                        let $a2 = rhs.as_float()?;
                        Ok(Value::from($operation))
                    } else {
                        let $a1 = self.as_int()?;
                        let $a2 = rhs.as_int()?;
                        Ok(Value::from($operation))
                    }
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
            Type::Int(left) => match &other.t {
                Type::Int(right) => left == right,
                Type::Float(right) => *left as f32 == *right,
                _ => false,
            },
            Type::Float(left) => match &other.t {
                Type::Float(right) => left == right,
                Type::Int(right) => *left == *right as f32,
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
            Type::Int(left) => match other.t {
                Type::Int(right) => left.partial_cmp(&right),
                Type::Float(right) => (*left as f32).partial_cmp(&right),
                _ => None,
            },
            Type::Float(left) => match other.t {
                Type::Float(right) => left.partial_cmp(&right),
                Type::Int(right) => left.partial_cmp(&(right as f32)),
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
            Type::Int(l) => write!(f, "{l}"),
            Type::Float(l) => {
                if l.fract() == 0. {
                    write!(f, "{l:.1}")
                } else {
                    write!(f, "{l}")
                }
            }
            Type::Str(s) => write!(f, "\"{s}\""),
        }
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self {
            t: Type::Int(value),
        }
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self {
            t: Type::Float(value),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self {
            t: Type::Int(if value { 1 } else { 0 }),
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
