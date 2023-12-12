use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

use crate::utils::error::{night_err, NightError, Status};
use crate::utils::function::BiFunction;
use crate::value::Value;

#[derive(Clone)]
pub enum StackVal {
    // TODO: See `interpreter.rs`'s `Instr::PushFunc` for message
    Function(BiFunction),
    Value(Value),
}

impl StackVal {
    pub fn as_fn(self) -> Status<BiFunction> {
        match self {
            Self::Function(f) => Ok(f),
            Self::Value(_) => night_err!(UnsupportedType, "Expected function, got value"),
        }
    }

    pub fn as_value(self) -> Status<Value> {
        match self {
            Self::Function(_) => night_err!(UnsupportedType, "Expected value, got function"),
            Self::Value(v) => Ok(v),
        }
    }
}

impl Display for StackVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function(def) => write!(f, "<function>: {:?}", def.instrs),
            Self::Value(v) => write!(f, "{v}"),
        }
    }
}

impl From<Value> for StackVal {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

impl From<BiFunction> for StackVal {
    fn from(value: BiFunction) -> Self {
        Self::Function(value)
    }
}

pub type Scope = std::rc::Rc<std::cell::RefCell<ScopeInternal>>;

#[derive(Clone, PartialEq, Eq, Hash)]
enum SymbolType {
    Symbol(String),
    Register(String),
}

impl ToString for SymbolType {
    fn to_string(&self) -> String {
        match self {
            Self::Symbol(s) => s.clone(),
            Self::Register(s) => format!("${s}"),
        }
    }
}

#[derive(Clone)]
pub struct ScopeInternal {
    stack: Vec<StackVal>,
    guard: HashSet<String>,
    block: HashSet<String>,
    env: HashMap<SymbolType, StackVal>,
}

impl ScopeInternal {
    pub fn create() -> Self {
        Self {
            stack: Vec::new(),
            guard: HashSet::new(),
            block: HashSet::new(),
            env: HashMap::new(),
        }
    }

    pub fn add_guard(&mut self, g: String) -> Status {
        if self.guard.insert(g.clone()) {
            Ok(())
        } else {
            night_err!(
                Runtime,
                format!("Attempted to guard register '${g}' when it was already guarded.")
            )
        }
    }

    pub fn rem_guard(&mut self, g: String) {
        self.guard.remove(&g);
        // `GuardEnd` always follows `Guard`, so the register will always be defined
        let _ = self.undef_reg(g);
    }

    pub fn add_block(&mut self, g: String) -> Status {
        if !self.guard.contains(&g) {
            night_err!(
                Runtime,
                format!("Cannot block register '${g}' when it is not guarded.")
            )
        } else if self.block.insert(g.clone()) {
            Ok(())
        } else {
            night_err!(
                Runtime,
                format!("Attempted to block register '${g}' when it was already blocked.")
            )
        }
    }

    pub fn rem_block(&mut self, g: String) {
        self.block.remove(&g);
    }

    pub fn pop(&mut self) -> Status<StackVal> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            _ => night_err!(NothingToPop),
        }
    }

    pub fn pop_value(&mut self) -> Status<Value> {
        match self.stack.pop() {
            Some(StackVal::Value(v)) => Ok(v),
            Some(_) => night_err!(UnsupportedType, "Expected literal value, found function."),
            _ => night_err!(NothingToPop),
        }
    }

    pub fn push(&mut self, val: StackVal) {
        self.stack.push(val);
    }

    pub fn push_value(&mut self, val: Value) {
        self.stack.push(StackVal::Value(val));
    }

    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    pub fn def_sym(&mut self, sym: String, s: StackVal) -> Status {
        let sym = SymbolType::Symbol(sym);
        if self.env.contains_key(&sym) {
            night_err!(SymbolRedefinition, sym.to_string())
        } else {
            self.env.insert(sym, s);
            Ok(())
        }
    }

    pub fn def_reg(&mut self, name: String, s: StackVal) -> Status<StackVal> {
        let guarded = self.guard.contains(&name);
        let reg = SymbolType::Register(name.clone());
        if self.env.contains_key(&reg) {
            if guarded {
                return night_err!(
                    Runtime,
                    format!("Register '${name}' is guarded, cannot redefine.")
                );
            }
            self.env.remove(&reg);
        }

        self.env.insert(reg, s.clone());
        Ok(s)
    }

    pub fn undef_sym(&mut self, sym: String) -> Status<StackVal> {
        let sym = SymbolType::Symbol(sym);
        self.env
            .remove(&sym)
            .ok_or(NightError::UndefinedSymbol(sym.to_string()))
    }

    pub fn undef_reg(&mut self, reg: String) -> Status {
        if self.guard.contains(&reg) {
            return night_err!(
                Runtime,
                format!("Register '${reg}' is guarded, cannot undefine.")
            );
        }
        let reg = SymbolType::Register(reg);
        self.env
            .remove(&reg)
            .map(|_| ())
            .ok_or(NightError::UndefinedSymbol(reg.to_string()))
    }

    pub fn get_sym(&self, sym: String) -> Status<&StackVal> {
        let sym = SymbolType::Symbol(sym);
        self.env
            .get(&sym)
            .ok_or(NightError::UndefinedSymbol(sym.to_string()))
    }

    pub fn get_reg(&self, reg: String) -> Status<&StackVal> {
        if self.block.contains(&reg) {
            return night_err!(
                Runtime,
                format!("Register '${reg}' is blocked, cannot access.")
            );
        }

        let reg = SymbolType::Register(reg);
        self.env
            .get(&reg)
            .ok_or(NightError::UndefinedSymbol(reg.to_string()))
    }

    pub fn raw_stack(&mut self) -> &mut Vec<StackVal> {
        &mut self.stack
    }
}

impl Display for ScopeInternal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for val in &self.stack {
            match val {
                StackVal::Value(v) => writeln!(f, "{v}")?,
                StackVal::Function(_) => writeln!(f, "<function>")?,
            }
        }

        Ok(())
    }
}

impl From<Vec<StackVal>> for ScopeInternal {
    fn from(value: Vec<StackVal>) -> Self {
        Self {
            stack: value,
            guard: HashSet::new(),
            block: HashSet::new(),
            env: HashMap::new(),
        }
    }
}
