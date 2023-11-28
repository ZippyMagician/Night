use std::collections::HashMap;
use std::fmt::{self, Display};
use std::rc::Rc;

use crate::utils::error::{NightError, Status};
use crate::utils::function::InlineFunction;
use crate::value::Value;

#[derive(Clone)]
pub enum ScopeEnv {
    Function(Rc<dyn InlineFunction>),
    Value(Value),
}

pub type Scope = std::rc::Rc<std::cell::RefCell<ScopeInternal>>;

#[derive(Clone)]
pub struct ScopeInternal {
    stack: Vec<ScopeEnv>,
    _env: HashMap<String, ScopeEnv>,
}

impl ScopeInternal {
    pub fn create() -> Self {
        Self {
            stack: Vec::new(),
            _env: HashMap::new(),
        }
    }

    pub fn pop(&mut self) -> Status<ScopeEnv> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            _ => Err(NightError::NothingToPop),
        }
    }

    pub fn pop_value(&mut self) -> Status<Value> {
        match self.stack.pop() {
            Some(ScopeEnv::Value(v)) => Ok(v),
            Some(_) => Err(NightError::UnsupportedType(
                "Expected literal value, found function.".to_string(),
            )),
            _ => Err(NightError::NothingToPop),
        }
    }

    pub fn push(&mut self, val: ScopeEnv) {
        self.stack.push(val);
    }

    pub fn push_value(&mut self, val: Value) {
        self.stack.push(ScopeEnv::Value(val));
    }
}

impl Display for ScopeInternal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for val in &self.stack {
            match val {
                ScopeEnv::Value(v) => writeln!(f, "{v}")?,
                ScopeEnv::Function(_) => writeln!(f, "<Function>")?,
            }
        }

        Ok(())
    }
}

impl From<Vec<ScopeEnv>> for ScopeInternal {
    fn from(value: Vec<ScopeEnv>) -> Self {
        Self {
            stack: value,
            _env: HashMap::new(),
        }
    }
}
