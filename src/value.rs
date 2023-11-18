use std::ops::Add;
use std::fmt::{self, Display};

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

impl Add for Value {
    type Output = Status<Value>;

    fn add(self, rhs: Self) -> Self::Output {
        match self.t {
            Type::Num(l) => match rhs.t {
                Type::Num(r) => Ok(Value::from(l + r)),
                _ => Err(NightError::UnsupportedType(
                    "Cannot add a number with a non-number.".to_string(),
                )),
            },
            _ => Err(NightError::UnsupportedType(
                "Cannot add a number with a non-number.".to_string(),
            )),
        }
    }
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
