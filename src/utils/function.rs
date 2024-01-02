use std::rc::Rc;

use crate::interpreter::Instr;
use crate::scope::{Scope, StackVal};
use crate::utils::error::Status;
use crate::value::Value;

/// Defines a struct that can generate a list of instructions to be executed
pub trait Generable {
    fn gen_instrs(&self, span: usize) -> Vec<Instr>;

    fn len(&self) -> usize;
}

#[derive(Clone)]
#[repr(transparent)]
pub struct BlockFunc {
    instrs: Vec<Instr>,
}

#[derive(Clone)]
pub struct CurriedFunc {
    op: StackVal,
    block: Rc<dyn Generable>,
}

#[derive(Clone)]
pub struct ComposedFunc {
    block1: Rc<dyn Generable>,
    block2: Rc<dyn Generable>,
}

#[derive(Clone)]
#[repr(transparent)]
pub struct SingleFunc(Instr);

impl Generable for BlockFunc {
    #[inline]
    fn gen_instrs(&self, _: usize) -> Vec<Instr> {
        self.instrs.clone()
    }

    #[inline]
    fn len(&self) -> usize {
        self.instrs.len()
    }
}

impl Generable for CurriedFunc {
    #[inline]
    fn gen_instrs(&self, span: usize) -> Vec<Instr> {
        let op = if let StackVal::Function(f) = &self.op {
            Instr::PushFunc(f.clone(), span)
        } else {
            Instr::Push(self.op.clone().as_value().unwrap(), span)
        };

        let mut s = Vec::with_capacity(self.len());
        s.push(op);
        s.extend(self.block.gen_instrs(span));
        s
    }

    #[inline]
    fn len(&self) -> usize {
        1 + self.block.len()
    }
}

impl Generable for ComposedFunc {
    #[inline]
    fn gen_instrs(&self, span: usize) -> Vec<Instr> {
        let mut s = Vec::with_capacity(self.len());
        s.extend(self.block1.gen_instrs(span));
        s.extend(self.block2.gen_instrs(span));
        s
    }

    #[inline]
    fn len(&self) -> usize {
        self.block1.len() + self.block2.len()
    }
}

impl Generable for SingleFunc {
    #[inline]
    fn gen_instrs(&self, _: usize) -> Vec<Instr> {
        vec![self.0.clone()]
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
        Self { op, block }
    }
}

impl ComposedFunc {
    #[inline]
    pub fn new(block1: Rc<dyn Generable>, block2: Rc<dyn Generable>) -> Self {
        Self { block1, block2 }
    }
}

impl From<Instr> for SingleFunc {
    #[inline]
    fn from(value: Instr) -> Self {
        Self(value)
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
