use phf::phf_map;

use super::{Builtin, Operator};
use crate::scope::Scope;
use crate::utils::error::{night_err, Status};
use crate::utils::function::{self, InlineFunction};
use crate::value::Value;

// TODO: fix this + create required `arity` functions
// not sure if this will be used for `define_builtins` as well. It probably will in
// some regard, although I don't think anything other than 0 or 1 output will be
// supported (since it doesn't fit well with how I handle error returns with `Status`.
macro_rules! _define_internal {
    // 1 arg 0 outputs
    ($scope:expr, 0(1), $def:expr) => {
        function::arity1_0($def, $scope)
    };
    // 1 arg 1 output
    ($scope:expr, 1(1), $def:expr) => {
        function::arity1_1($def, $scope)
    };
    // 2 args 1 output
    ($scope:expr, 1(2), $def:expr) => {
        function::arity2_1($def, $scope)
    };
    // all other cases
    // TODO: maybe other common patterns supported
    ($scope:expr, $a:tt($b:tt), $def:expr) => {
        ($def)($scope)
    };
}

macro_rules! define_ops {
    ($($rep:expr => ($tok:pat, $lit:expr, $a:tt($b:tt): $def:expr));*;) => {
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

            pub fn from_name(n: impl AsRef<str>) -> Option<Self> {
                $(
                    if $lit == n.as_ref() {
                        return OP_MAP.get($rep).copied();
                    }
                )*
                None
            }
        }

        impl InlineFunction for Operator {
            fn call(&self, scope: Scope) -> Status {
                match self {
                    $(
                        $tok => _define_internal!(scope, $a($b), $def).into()
                    ),*
                }
            }
        }
    }
}

macro_rules! define_builtins {
    ($($rep:expr => ($tok:pat, $a:tt($b:tt): $def:expr));*;) => {
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
                        $tok => _define_internal!(scope, $a($b), $def).into()
                    ),*
                }
            }
        }
    }
}

// Operator mappings for the tokens and their literal repr
define_ops! {
    "+" => (Operator::Add, "add", 1(2): op_add);

    "-" => (Operator::Sub, "sub", 1(2): op_sub);

    "/" => (Operator::Div, "div", 1(2): op_div);

    "*" => (Operator::Mul, "mul", 1(2): op_mul);

    "%" => (Operator::Mod, "mod", 1(2): op_mod);

    "=" => (Operator::Eq, "eq", 0(0): |_: Scope| Ok(()));

    "!=" => (Operator::Neq, "neq", 0(0): |_: Scope| Ok(()));

    ">" => (Operator::Gt, "gt", 0(0): |_: Scope| Ok(()));

    "<" => (Operator::Lt, "lt", 0(0): |_: Scope| Ok(()));

    ">=" => (Operator::Gte, "gte", 0(0): |_: Scope| Ok(()));

    "<=" => (Operator::Lte, "lte", 0(0): |_: Scope| Ok(()));

    "!" => (Operator::Assign, "tmpa", 0(0): |_: Scope| Ok(()));

    ";" => (Operator::Pop, "pop", 0(1): op_pop);

    ":" => (Operator::Swap, "swap", 2(2): op_swap);

    "." => (Operator::Dup, "dup", 2(1): op_dup);

    "?" => (Operator::Call, "call", 0(0): |_: Scope| {
        night_err!(Unimplemented, "An internal error occurred, this should not have been called")
    });
}

define_builtins! {
    "print" => (Builtin::Print, 0(1): |v: Value| {
        println!("{v}");
        Ok(())
    });

    "inc" => (Builtin::Inc, 1(1): |v: Value| {
        let n = v.as_num()?;
        Ok(Value::from(n + 1))
    });

    "dec" => (Builtin::Dec, 1(1): |v: Value| {
        let n = v.as_num()?;
        Ok(Value::from(n - 1))
    });

    "def" => (Builtin::Def, 0(2): |scope: Scope| {
        let mut s = scope.borrow_mut();
        let name = s.pop_value()?.as_str()?;
        let value = s.pop()?;
        s.def(name, value)
    });

    "undef" => (Builtin::Undef, 0(1): |_: Value| {
        night_err!(Unimplemented, "Symbol undefinition")
    });
}

fn op_add(left: Value, right: Value) -> Status<Value> {
    left + right
}

fn op_sub(left: Value, right: Value) -> Status<Value> {
    left - right
}

fn op_mul(left: Value, right: Value) -> Status<Value> {
    left * right
}

fn op_div(left: Value, right: Value) -> Status<Value> {
    left / right
}

fn op_mod(left: Value, right: Value) -> Status<Value> {
    left % right
}

fn op_pop(arg: Value) -> Status {
    drop(arg);
    Ok(())
}

fn op_swap(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let right = s.pop()?;
    let left = s.pop()?;
    s.push(right);
    s.push(left);
    Ok(())
}

fn op_dup(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let val = s.pop()?;
    s.push(val.clone());
    s.push(val);
    Ok(())
}
