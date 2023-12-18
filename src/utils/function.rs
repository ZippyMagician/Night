use crate::interpreter::Instr;
use crate::scope::{Scope, StackVal};
use crate::utils::error::Status;
use crate::value::Value;

/// Defines a struct that can generate a list of instructions to be executed
pub trait Generable {
    fn gen_instrs<'a>(&'a self) -> &'a [Instr];

    fn len(&self) -> usize;
}

#[derive(Clone, Debug)]
pub struct BlockFunc {
    instrs: Vec<Instr>,
}

impl Generable for BlockFunc {
    fn gen_instrs(&self) -> &[Instr] {
        &self.instrs
    }

    fn len(&self) -> usize {
        self.instrs.len()
    }
}

impl<T> From<T> for BlockFunc
where
    T: Into<Vec<Instr>>,
{
    fn from(value: T) -> Self {
        Self {
            instrs: value.into(),
        }
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
