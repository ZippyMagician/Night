use std::collections::HashMap;
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

    pub fn pop(&mut self) -> Status {
        Err(NightError::Unimplemented(
            "ScopeInternal::pop unimplemented".to_string(),
        ))
    }

    pub fn pop_value(&mut self) -> Status<Value> {
        match self.stack.pop() {
            Some(ScopeEnv::Value(v)) => Ok(v),
            _ => Err(NightError::NothingToPop),
        }
    }

    pub fn push(&mut self, val: ScopeEnv) {
        self.stack.push(val);
    }

    pub fn push_val(&mut self, val: Value) {
        self.stack.push(ScopeEnv::Value(val));
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
