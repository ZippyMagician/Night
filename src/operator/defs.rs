use phf::phf_map;

use crate::scope::Scope;
use crate::utils::error::Status;
use crate::utils::function::InlineFunction;
use super::Operator;

macro_rules! define {
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
            fn call(&mut self, scope: Scope) -> Status {
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
define! {
    "+" => (Operator::Add, "add", |s: Scope| Status::pass(s));

    "-" => (Operator::Sub, "sub", |s: Scope| Status::pass(s));

    "/" => (Operator::Div, "div", |s: Scope| Status::pass(s));

    "*" => (Operator::Mul, "mul", |s: Scope| Status::pass(s));

    "=" => (Operator::Eq, "eq", |s: Scope| Status::pass(s));

    "!=" => (Operator::Neq, "neq", |s: Scope| Status::pass(s));

    ">" => (Operator::Gt, "gt", |s: Scope| Status::pass(s));

    "<" => (Operator::Lt, "lt", |s: Scope| Status::pass(s));

    ">=" => (Operator::Gte, "gte", |s: Scope| Status::pass(s));

    "<=" => (Operator::Lte, "lte", |s: Scope| Status::pass(s));

    "!" => (Operator::Assign, "tmpa", |s: Scope| Status::pass(s));

    ";" => (Operator::Pop, "pop", |s: Scope| Status::pass(s));

    ":" => (Operator::Swap, "swap", |s: Scope| Status::pass(s));

    "." => (Operator::Dup, "dup", |s: Scope| Status::pass(s));

    "?" => (Operator::Call, "call", |s: Scope| Status::pass(s))
}
