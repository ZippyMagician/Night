use std::ops::Add;

use phf::phf_map;

use super::{Builtin, Operator};
use crate::scope::Scope;
use crate::utils::error::Status;
use crate::utils::function::InlineFunction;

macro_rules! define_ops {
    ($($rep:expr => ($tok:pat, $lit:expr, $def:expr));*) => {
        pub static OP_MAP: phf::Map<&'static str, Operator> = phf_map! {
            $(
               $rep => $tok
            ),*
        };

        impl Operator {
            pub fn get_glyph(&self) -> &'static str {
                match self {
                    $(
                        $tok => $rep
                    ),*
                }
            }

            pub fn name(&self) -> &'static str {
                match self {
                    $(
                        $tok => $lit
                    ),*
                }
            }
        }

        impl InlineFunction for Operator {
            fn call(&self, scope: Scope) -> Status {
                match self {
                    $(
                        $tok => ($def)(scope)
                    ),*
                }
            }
        }
    }
}

macro_rules! define_builtins {
    ($($rep:expr => ($tok:pat, $def:expr)),*) => {
        pub static BUILTIN_MAP: phf::Map<&'static str, Builtin> = phf_map! {
            $(
                $rep => $tok
            ),*
        };

        impl Builtin {
            pub fn name(&self) -> &'static str {
                match self {
                    $(
                        $tok => $rep
                    ),*
                }
            }
        }

        impl InlineFunction for Builtin {
            fn call(&self, scope: Scope) -> Status {
                match self {
                    $(
                        $tok => ($def)(scope)
                    ),*
                }
            }
        }
    }
}

// Operator mappings for the tokens and their literal repr
define_ops! {
    "+" => (Operator::Add, "add", op_add);

    "-" => (Operator::Sub, "sub", |_: Scope| Ok(()));

    "/" => (Operator::Div, "div", |_: Scope| Ok(()));

    "*" => (Operator::Mul, "mul", |_: Scope| Ok(()));

    "=" => (Operator::Eq, "eq", |_: Scope| Ok(()));

    "!=" => (Operator::Neq, "neq", |_: Scope| Ok(()));

    ">" => (Operator::Gt, "gt", |_: Scope| Ok(()));

    "<" => (Operator::Lt, "lt", |_: Scope| Ok(()));

    ">=" => (Operator::Gte, "gte", |_: Scope| Ok(()));

    "<=" => (Operator::Lte, "lte", |_: Scope| Ok(()));

    "!" => (Operator::Assign, "tmpa", |_: Scope| Ok(()));

    ";" => (Operator::Pop, "pop", |_: Scope| Ok(()));

    ":" => (Operator::Swap, "swap", |_: Scope| Ok(()));

    "." => (Operator::Dup, "dup", |_: Scope| Ok(()));

    "?" => (Operator::Call, "call", |_: Scope| Ok(()))
}

define_builtins! {
    "print" => (Builtin::Print, |scope: Scope| {
        let mut scope = scope.borrow_mut();
        let val = scope.pop_value()?;
        println!("{val}");
        Ok(())
    })
}

fn op_add(scope: Scope) -> Status {
    let mut scope = scope.borrow_mut();
    let r = scope.pop_value()?;
    let l = scope.pop_value()?;
    scope.push_val(l.add(r)?);
    Ok(())
}
