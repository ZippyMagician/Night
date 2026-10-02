use std::collections::{HashMap, HashSet};
use std::fmt::{self, Debug, Display};
use std::rc::Rc;

use crate::utils::error::{night_err, NightError, Status};
use crate::utils::function::Generable;
use crate::value::Value;

#[derive(Clone)]
pub enum StackVal {
    // TODO: See `interpreter.rs`'s `Instr::PushFunc` for message
    Function(Rc<dyn Generable>),
    Value(Value),
}

impl StackVal {
    pub fn as_fn(self) -> Status<Rc<dyn Generable>> {
        match self {
            Self::Function(f) => Ok(f),
            Self::Value(_) => night_err!(UnsupportedType, "Expected function, got literal value"),
        }
    }

    pub fn as_value(self) -> Status<Value> {
        match self {
            Self::Function(_) => {
                night_err!(UnsupportedType, "Expected literal value, got function")
            }
            Self::Value(v) => Ok(v),
        }
    }

    #[inline]
    pub fn is_str(&self) -> bool {
        match &self {
            Self::Value(v) => v.is_str(),
            _ => false,
        }
    }

    #[inline]
    pub fn as_str_unchecked(self) -> String {
        if let Self::Value(v) = self {
            v.as_str_unchecked()
        } else {
            unreachable!();
        }
    }
}

impl Debug for StackVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function(_) => write!(f, "<function>"),
            Self::Value(v) => write!(f, "{v:?}"),
        }
    }
}

impl Display for StackVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function(func) => write!(f, "{func}"),
            Self::Value(v) => write!(f, "{v}"),
        }
    }
}

impl PartialEq for StackVal {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Value(v) if let Self::Value(r) = other => v == r,
            _ => false,
        }
    }
}

impl PartialOrd for StackVal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match &self {
            Self::Value(v) if let Self::Value(r) = other => v.partial_cmp(r),
            _ => None,
        }
    }
}

impl From<Value> for StackVal {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

impl From<Rc<dyn Generable>> for StackVal {
    fn from(value: Rc<dyn Generable>) -> Self {
        Self::Function(value)
    }
}

impl<T> From<T> for StackVal
where
    T: Generable + 'static,
{
    fn from(value: T) -> Self {
        Self::Function(Rc::new(value))
    }
}

pub type Scope = std::rc::Rc<std::cell::RefCell<ScopeInternal>>;

#[derive(Clone)]
struct RegTrace {
    guarded: bool,
    trace: Vec<StackVal>,
}

impl RegTrace {
    pub fn new() -> Self {
        Self {
            guarded: false,
            trace: vec![],
        }
    }

    pub fn is_guarded(&self) -> bool {
        self.guarded
    }

    pub fn guard(&mut self) {
        self.guarded = true;
    }

    pub fn has_trace(&self) -> bool {
        !self.trace.is_empty()
    }

    pub fn push(&mut self, val: StackVal) -> Status {
        if !self.guarded {
            return night_err!(
                Runtime,
                "Cannot override guarded register unless a parent guard exists."
            );
        }
        self.guarded = false;
        self.trace.push(val);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<StackVal> {
        self.trace.pop()
    }
}

#[derive(Clone)]
pub struct ScopeInternal {
    stack: Vec<StackVal>,
    guard: HashSet<String>,
    block: HashSet<String>,
    env_sym: HashMap<String, StackVal>,
    env_reg: HashMap<String, StackVal>,
    register_trace: HashMap<String, RegTrace>,
}

impl ScopeInternal {
    pub fn create() -> Self {
        Self {
            stack: Vec::new(),
            guard: HashSet::new(),
            block: HashSet::new(),
            env_sym: HashMap::new(),
            env_reg: HashMap::new(),
            register_trace: HashMap::new(),
        }
    }

    fn add_trace(&mut self, g: String) {
        if !self.register_trace.contains_key(&g) {
            self.register_trace.insert(g.clone(), RegTrace::new());
        }
    }

    pub fn dump_symbols(&self) {
        println!("--- SYMBOL DMP: ---");
        for (key, val) in &self.env_sym {
            println!("{key}: {val}");
        }
        println!("-------------------");
    }

    // For now this returns a `Status`, as I might use it in the future.
    pub fn add_guard(&mut self, g: String) -> Status {
        if !self.guard.insert(g.clone()) {
            self.add_trace(g.clone());
            let trace = self.register_trace.get_mut(&g).unwrap();
            trace.guard();

            // If a register is blocked, but a new guard overrides it, it is fine to remove the block early
            self.block.remove(&g);
            if let Some(v) = self.env_reg.remove(&g) {
                trace.push(v)?;
            }
        } else if self.env_reg.contains_key(&g) {
            // The register is newly guarded, but has a previous value assigned to it
            self.env_reg.remove(&g);
            return night_err!(
                Warning,
                format!("Global register '${g}' will be overwritten due to guard statement.")
            );
        }

        if self.register_trace.get(&g).is_none() {
            self.add_trace(g);
        }
        Ok(())
    }

    pub fn rem_guard(&mut self, g: String) -> Status {
        let trace = self.register_trace.get(&g);
        if trace.map(|n| !n.has_trace()).unwrap_or(true) {
            self.guard.remove(&g);
        }
        self.undef_reg(g)
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

    pub fn pop2(&mut self) -> Status<(StackVal, StackVal)> {
        let top = self.pop()?;
        Ok((self.pop()?, top))
    }

    pub fn pop3(&mut self) -> Status<(StackVal, StackVal, StackVal)> {
        let c = self.pop()?;
        let b = self.pop()?;
        Ok((self.pop()?, b, c))
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

    pub fn push_all<const C: usize>(&mut self, vals: [StackVal; C]) {
        for val in vals {
            self.stack.push(val);
        }
    }

    pub fn push_value(&mut self, val: Value) {
        self.stack.push(StackVal::Value(val));
    }

    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    pub fn def_sym(&mut self, sym: String, s: StackVal) -> Status {
        if self.env_sym.contains_key(&sym) {
            night_err!(SymbolRedefinition, sym.to_string())
        } else {
            self.env_sym.insert(sym, s);
            Ok(())
        }
    }

    pub fn def_reg(&mut self, name: String, s: StackVal) -> Status<StackVal> {
        self.add_trace(name.clone());
        let trace = self.register_trace.get_mut(&name).unwrap();

        let guarded = self.guard.contains(&name);
        if self.env_reg.contains_key(&name) {
            if guarded && !trace.is_guarded() {
                return night_err!(
                    Runtime,
                    format!("Register '${name}' is guarded, cannot redefine.")
                );
            } else if guarded && trace.is_guarded() {
                let v = self.env_reg.remove(&name).unwrap();
                trace.push(v)?;
            } else {
                self.env_reg.remove(&name);
            }
        }

        self.env_reg.insert(name, s.clone());
        Ok(s)
    }

    pub fn undef_sym(&mut self, sym: String) -> Status<StackVal> {
        self.env_sym
            .remove(&sym)
            .ok_or(NightError::UndefinedSymbol(sym))
    }

    pub fn undef_reg(&mut self, name: String) -> Status {
        let trace = self.register_trace.get_mut(&name).unwrap();
        if self.guard.contains(&name) && !trace.has_trace() {
            return night_err!(
                Runtime,
                format!("Register '${name}' is guarded, cannot undefine.")
            );
        }
        self.env_reg
            .remove(&name)
            .map(|_| ())
            .ok_or(NightError::Runtime(format!(
                "Guarded register '${name}' must be defined within the block."
            )))?;

        // Reset register to previous guarded value
        if let Some(v) = trace.pop() {
            self.env_reg.insert(name, v);
        }
        Ok(())
    }

    pub fn get_sym(&self, sym: &str) -> Status<&StackVal> {
        self.env_sym
            .get(sym)
            .ok_or(NightError::UndefinedSymbol(sym.to_string()))
    }

    pub fn get_reg(&self, reg: &str) -> Status<&StackVal> {
        if self.block.contains(reg) {
            return night_err!(
                Runtime,
                format!("Register '${reg}' is blocked, cannot access.")
            );
        }

        self.env_reg
            .get(reg)
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
            env_sym: HashMap::new(),
            env_reg: HashMap::new(),
            register_trace: HashMap::new(),
        }
    }
}
