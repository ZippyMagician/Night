use crate::scope::Scope;
use crate::utils::error::Status;
use crate::value::Value;

// Function trait for built-in symbols and operators
pub trait InlineFunction {
    fn call(&self, scope: Scope) -> Status;
}

impl InlineFunction for fn(Scope) -> Status {
    fn call(&self, scope: Scope) -> Status {
        self(scope)
    }
}

pub fn arity1_0(def: fn(Value) -> Status, scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let arg = s.pop_value()?;
    def(arg)
}

pub fn arity1_1(def: fn(Value) -> Status<Value>, scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let arg = s.pop_value()?;
    s.push_value(def(arg)?);
    Ok(())
}

pub fn arity2_1(def: fn(Value, Value) -> Status<Value>, scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let right = s.pop_value()?;
    let left = s.pop_value()?;
    s.push_value(def(left, right)?);
    Ok(())
}
