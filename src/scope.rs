use std::collections::HashMap;
use std::fmt::{self, Display};

use crate::utils::error::{night_err, NightError, Status};
use crate::utils::function::BiFunction;
use crate::value::Value;

#[derive(Clone)]
pub enum ScopeEnv {
    // TODO: See `interpreter.rs`'s `Instr::PushFunc` for message
    Function(BiFunction),
    Value(Value),
}

pub type Scope = std::rc::Rc<std::cell::RefCell<ScopeInternal>>;

#[derive(Clone)]
pub struct ScopeInternal {
    stack: Vec<ScopeEnv>,
    env: HashMap<String, ScopeEnv>,
}

impl ScopeInternal {
    pub fn create() -> Self {
        Self {
            stack: Vec::new(),
            env: HashMap::new(),
        }
    }

    pub fn pop(&mut self) -> Status<ScopeEnv> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            _ => night_err!(NothingToPop),
        }
    }

    pub fn pop_value(&mut self) -> Status<Value> {
        match self.stack.pop() {
            Some(ScopeEnv::Value(v)) => Ok(v),
            Some(_) => night_err!(UnsupportedType, "Expected literal value, found function."),
            _ => night_err!(NothingToPop),
        }
    }

    pub fn push(&mut self, val: ScopeEnv) {
        self.stack.push(val);
    }

    pub fn push_value(&mut self, val: Value) {
        self.stack.push(ScopeEnv::Value(val));
    }

    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    pub fn def(&mut self, sym: String, s: ScopeEnv) -> Status {
        if self.env.contains_key(&sym) {
            night_err!(SymbolRedefinition, sym)
        } else {
            self.env.insert(sym, s);
            Ok(())
        }
    }

    pub fn get_def(&self, sym: String) -> Status<&ScopeEnv> {
        self.env.get(&sym).ok_or(NightError::UndefinedSymbol(sym))
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
            env: HashMap::new(),
        }
    }
}
