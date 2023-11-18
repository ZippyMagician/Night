use crate::scope::Scope;
use crate::utils::error::Status;

pub trait InlineFunction {
    fn call(&mut self, scope: Scope) -> Status<Scope>;
}
