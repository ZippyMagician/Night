use crate::scope::Scope;
use crate::utils::error::Status;

// Function trait for built-in symbols and operators
pub trait InlineFunction {
    fn call(&self, scope: Scope) -> Status;
}
