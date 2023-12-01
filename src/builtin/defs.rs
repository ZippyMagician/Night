use phf::phf_map;

use super::{Builtin, Operator};
use crate::scope::{Scope, StackVal};
use crate::utils;
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
    // 0 arg 1 output
    ($scope:expr, 1(0), $def:expr) => {
        function::arity0_1($def, $scope)
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

    "!" => (Operator::Assign, "defr", 1(0): op_defr);

    ";" => (Operator::Pop, "pop", 0(0): op_pop);

    ":" => (Operator::Swap, "swp", 2(2): op_swap);

    "." => (Operator::Dup, "dup", 2(1): op_dup);

    "?" => (Operator::Call, "call", 0(0): |_: Scope| {
        night_err!(ContextFail, "An internal error occurred, this should not have been called")
    });
}

fn op_add(_: Scope, left: Value, right: Value) -> Status<Value> {
    left + right
}

fn op_sub(_: Scope, left: Value, right: Value) -> Status<Value> {
    left - right
}

fn op_mul(_: Scope, left: Value, right: Value) -> Status<Value> {
    left * right
}

fn op_div(_: Scope, left: Value, right: Value) -> Status<Value> {
    left / right
}

fn op_mod(_: Scope, left: Value, right: Value) -> Status<Value> {
    left % right
}

fn op_pop(scope: Scope) -> Status {
    scope.borrow_mut().pop()?;
    Ok(())
}

fn op_defr(scope: Scope) -> Status<StackVal> {
    let mut s = scope.borrow_mut();
    let name = s.pop_value()?.as_str()?;
    let value = s.pop()?;
    s.def_reg(name, value)
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

define_builtins! {
    "print" => (Builtin::Print, 0(0): |scope: Scope| {
        let v = scope.borrow_mut().pop()?;
        println!("{v}");
        Ok(())
    });

    "inc" => (Builtin::Inc, 1(1): |_, v| {
        let n = v.as_num()?;
        Ok(Value::from(n + 1))
    });

    "dec" => (Builtin::Dec, 1(1): |_, v| {
        let n = v.as_num()?;
        Ok(Value::from(n - 1))
    });

    "def" => (Builtin::Def, 0(2): |scope: Scope| {
        let mut s = scope.borrow_mut();
        let name = s.pop_value()?.as_str()?;
        if !utils::is_one_word(&name) {
            return night_err!(Runtime, format!("'{name}' is not a valid symbol name."))
        }
        let value = s.pop()?;
        s.def_sym(name, value)
    });

    "undef" => (Builtin::Undef, 1(1): |scope, name| {
        let mut s = scope.borrow_mut();
        let name = name.as_str()?;
        if !utils::is_one_word(&name) {
            return night_err!(Runtime, format!("'${name}' is not a valid symbol name."))
        }
        s.undef_sym(name)
    });

    "undefr" => (Builtin::UndefReg, 0(1): |scope, name| {
        let mut s = scope.borrow_mut();
        let name = name.as_str()?;
        if !utils::is_one_word(&name) {
            return night_err!(Runtime, format!("'${name}' is not a valid register name."))
        }
        s.undef_reg(name)
    });

    "loop" => (Builtin::Loop, 0(0): |_| {
        night_err!(ContextFail, "An internal error occurred, this should not have been called")
    });
}
