use super::scope::Scope;

pub trait InlineFunction {
    fn call(&self, scope: Scope) -> Result<Scope, Box<dyn ToString>>;
}

/* Example of function
impl InlineFunction for super::token::OpName::Add {
    pub fn call(&self, mut scope: Scope) -> Result<Scope, impl ToString> {
        let l = scope.pop_stack()?;
        let r = scope.pop_stack()?;
        scope.push_stack(Value::from(l.as_num()? + r.as_num()?));
        Ok(scope)
    }
}
*/

pub enum Op {}
