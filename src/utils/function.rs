use std::fmt;
use std::rc::Rc;

use crate::interpreter::Instr;
use crate::scope::{Scope, StackVal};
use crate::utils::error::Status;
use crate::value::Value;

/// Defines a struct that can generate a list of instructions to be executed
pub trait Generable {
    fn gen_instrs(&self) -> &[Instr];

    fn len(&self) -> usize;
}

#[derive(Clone)]
#[repr(transparent)]
pub struct BlockFunc {
    instrs: Vec<Instr>,
}

#[derive(Clone)]
pub struct CurriedFunc {
    //op: StackVal,
    //block: Rc<dyn Generable>,
    instrs: Vec<Instr>,
}

#[derive(Clone)]
pub struct ComposedFunc {
    //block1: Rc<dyn Generable>,
    //block2: Rc<dyn Generable>,
    instrs: Vec<Instr>,
}

#[derive(Clone)]
#[repr(transparent)]
pub struct SingleFunc([Instr; 1]);

impl Generable for BlockFunc {
    #[inline]
    fn gen_instrs(&self) -> &[Instr] {
        &self.instrs
    }

    #[inline]
    fn len(&self) -> usize {
        self.instrs.len()
    }
}

impl Generable for CurriedFunc {
    #[inline]
    fn gen_instrs(&self) -> &[Instr] {
        &self.instrs
    }

    #[inline]
    fn len(&self) -> usize {
        self.instrs.len()
    }
}

impl Generable for ComposedFunc {
    #[inline]
    fn gen_instrs(&self) -> &[Instr] {
        &self.instrs
    }

    #[inline]
    fn len(&self) -> usize {
        self.instrs.len()
    }
}

impl Generable for SingleFunc {
    #[inline]
    fn gen_instrs(&self) -> &[Instr] {
        &self.0
    }

    #[inline]
    fn len(&self) -> usize {
        1
    }
}

impl<T> From<T> for BlockFunc
where
    T: Into<Vec<Instr>>,
{
    #[inline]
    fn from(value: T) -> Self {
        Self {
            instrs: value.into(),
        }
    }
}

impl CurriedFunc {
    #[inline]
    pub fn new(op: StackVal, block: Rc<dyn Generable>) -> Self {
        let v = block.gen_instrs();
        let mut s = Vec::with_capacity(v.len() + 1);
        s.push(if let StackVal::Function(f) = op {
            Instr::PushFunc(f.clone(), usize::MAX)
        } else {
            Instr::Push(op.as_value().unwrap(), usize::MAX)
        });
        s.extend(v.iter().cloned());
        Self { instrs: s }
    }
}

impl ComposedFunc {
    #[inline]
    pub fn new(block1: Rc<dyn Generable>, block2: Rc<dyn Generable>) -> Self {
        let b1 = block1.gen_instrs();
        let b2 = block2.gen_instrs();
        let mut s = Vec::with_capacity(b1.len() + b2.len());
        s.extend(b1.iter().cloned());
        s.extend(b2.iter().cloned());
        Self { instrs: s }
    }
}

impl From<Instr> for SingleFunc {
    #[inline]
    fn from(value: Instr) -> Self {
        Self([value])
    }
}

impl fmt::Display for dyn Generable {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let instrs = self.gen_instrs();
        write!(f, "{{ ")?;
        for instr in instrs {
            write!(f, "{instr} ")?;
        }
        write!(f, "}}")
    }
}

#[inline]
pub fn arity0_1<T>(def: fn(Scope) -> Status<T>, scope: Scope) -> Status
where
    T: Into<StackVal>,
{
    let v = def(scope.clone())?.into();
    scope.borrow_mut().push(v);
    Ok(())
}

#[inline]
pub fn arity1_0(def: fn(Scope, Value) -> Status, scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let arg = s.pop_value()?;
    drop(s);
    def(scope, arg)
}

#[inline]
pub fn arity1_1<T>(def: fn(Scope, Value) -> Status<T>, scope: Scope) -> Status
where
    T: Into<StackVal>,
{
    let arg = scope.borrow_mut().pop_value()?;
    let v = def(scope.clone(), arg)?.into();
    scope.borrow_mut().push(v);
    Ok(())
}

#[inline]
pub fn arity2_1<T>(def: fn(Scope, Value, Value) -> Status<T>, scope: Scope) -> Status
where
    T: Into<StackVal>,
{
    let mut s = scope.borrow_mut();
    let right = s.pop_value()?;
    let left = s.pop_value()?;
    drop(s);
    let v = def(scope.clone(), left, right)?.into();
    scope.borrow_mut().push(v);
    Ok(())
}
