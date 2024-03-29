use std::rc::Rc;

use phf::phf_map;

use super::{Builtin, Operator};
use crate::scope::{Scope, StackVal};
use crate::utils;
use crate::utils::error::{night_err, Status};
use crate::utils::function::{self, ComposedFunc, CurriedFunc};
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

            pub fn call(&self, scope: Scope) -> Status {
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

            pub fn call(&self, scope: Scope) -> Status {
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

    "=" => (Operator::Eq, "eq", 1(2): op_eq);

    "!=" => (Operator::NotEq, "neq", 1(2): op_neq);

    ">" => (Operator::Greater, "gt", 1(2): op_gt);

    "<" => (Operator::Less, "lt", 1(2): op_lt);

    ">=" => (Operator::GreaterEq, "gte", 1(2): op_gte);

    "<=" => (Operator::LessEq, "lte", 1(2): op_lte);

    "~" => (Operator::Not, "not", 1(1): op_not);

    ";" => (Operator::Pop, "pop", 0(0): op_pop);

    ":" => (Operator::Swap, "swp", 2(2): op_swap);

    "." => (Operator::Dup, "dup", 2(1): op_dup);

    "?" => (Operator::Call, "call", 0(0): |_: Scope| {
        night_err!(ContextFail, "An internal error occurred, this should not have been called.")
    });
}

define_builtins! {
    "print" => (Builtin::Print, 0(0): print);

    "inc" => (Builtin::Inc, 1(1): inc);

    "dec" => (Builtin::Dec, 1(1): dec);

    "def" => (Builtin::Def, 0(2): def);

    "undef" => (Builtin::Undef, 1(1): undef);

    "undefr" => (Builtin::UndefReg, 0(1): undefr);

    "over" => (Builtin::Over, 3(2): over);

    "rot" => (Builtin::Rot, 0(0): rot);

    "rotr" => (Builtin::RotRight, 0(0): rotr);

    "dupd" => (Builtin::Dupd, 3(2): dup_dip);

    "swpd" => (Builtin::Swapd, 3(3): swap_dip);

    "pop2" => (Builtin::Pop2, 0(2): pop2);

    "pop3" => (Builtin::Pop3, 0(3): pop3);

    "nip" => (Builtin::Popd, 0(0): pop_dip);

    "dup2" => (Builtin::Dup2, 4(2): dup2);

    "dup3" => (Builtin::Dup3, 6(3): dup3);

    "pick" => (Builtin::Pick, 4(3): pick);

    "and" => (Builtin::LogicalAnd, 1(2): logical_and);

    "or" => (Builtin::LogicalOr, 1(2): logical_or);

    "floor" => (Builtin::Floor, 1(1): floor);

    "ceil" => (Builtin::Ceil, 1(1): ceil);

    "i64" => (Builtin::CastToInt, 1(1): cast_to_int);

    "f64" => (Builtin::CastToFloat, 1(1): cast_to_float);

    "curry" => (Builtin::Curry, 0(0): curry);

    "bind" => (Builtin::Bind, 0(0): bind);
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

fn op_eq(_: Scope, left: Value, right: Value) -> Status<Value> {
    Ok(Value::from(left == right))
}

fn op_neq(_: Scope, left: Value, right: Value) -> Status<Value> {
    Ok(Value::from(left != right))
}

fn op_gt(_: Scope, left: Value, right: Value) -> Status<Value> {
    if Value::types_match(&left, &right) {
        Ok(Value::from(left > right))
    } else {
        night_err!(
            UnsupportedType,
            format!("Cannot order '{left}' and '{right}' as their types do not match.")
        )
    }
}

fn op_lt(_: Scope, left: Value, right: Value) -> Status<Value> {
    if Value::types_match(&left, &right) {
        Ok(Value::from(left < right))
    } else {
        night_err!(
            UnsupportedType,
            format!("Cannot order '{left}' and '{right}' as their types do not match.")
        )
    }
}

fn op_gte(_: Scope, left: Value, right: Value) -> Status<Value> {
    if Value::types_match(&left, &right) {
        Ok(Value::from(left >= right))
    } else {
        night_err!(
            UnsupportedType,
            format!("Cannot order '{left}' and '{right}' as their types do not match.")
        )
    }
}

fn op_lte(_: Scope, left: Value, right: Value) -> Status<Value> {
    if Value::types_match(&left, &right) {
        Ok(Value::from(left <= right))
    } else {
        night_err!(
            UnsupportedType,
            format!("Cannot order '{left}' and '{right}' as their types do not match.")
        )
    }
}

fn op_not(_: Scope, value: Value) -> Status<Value> {
    Ok(Value::from(!value.as_bool()?))
}

fn op_pop(scope: Scope) -> Status {
    scope.borrow_mut().pop()?;
    Ok(())
}

fn op_swap(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let (left, right) = s.pop2()?;
    s.push_all([right, left]);
    Ok(())
}

fn op_dup(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let val = s.pop()?;
    s.push_all([val.clone(), val]);
    Ok(())
}

fn print(scope: Scope) -> Status {
    let v = scope.borrow_mut().pop()?;
    println!("{v}");
    Ok(())
}

fn inc(_: Scope, v: Value) -> Status<Value> {
    v + Value::from(1)
}

fn dec(_: Scope, v: Value) -> Status<Value> {
    v - Value::from(1)
}

fn def(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let name = s.pop_value()?.as_str()?;
    if !utils::is_one_word(&name) {
        return night_err!(Runtime, format!("'{name}' is not a valid symbol name."));
    }
    let value = s.pop()?;
    s.def_sym(name, value)
}

fn undef(scope: Scope, name: Value) -> Status<StackVal> {
    let mut s = scope.borrow_mut();
    let name = name.as_str()?;
    if !utils::is_one_word(&name) {
        return night_err!(Runtime, format!("'${name}' is not a valid symbol name."));
    }
    s.undef_sym(name)
}

fn undefr(scope: Scope, name: Value) -> Status {
    let mut s = scope.borrow_mut();
    let name = name.as_str()?;
    if !utils::is_one_word(&name) {
        return night_err!(Runtime, format!("'${name}' is not a valid register name."));
    }
    s.undef_reg(name)
}

fn over(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let (bottom, top) = s.pop2()?;
    s.push_all([bottom.clone(), top, bottom]);
    Ok(())
}

fn rot(scope: Scope) -> Status {
    scope.borrow_mut().raw_stack().rotate_left(1);
    Ok(())
}

fn rotr(scope: Scope) -> Status {
    scope.borrow_mut().raw_stack().rotate_right(1);
    Ok(())
}

fn dup_dip(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let (x, top) = s.pop2()?;
    s.push_all([x.clone(), x, top]);
    Ok(())
}

fn swap_dip(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let (left, right, top) = s.pop3()?;
    s.push_all([right, left, top]);
    Ok(())
}

fn pop2(scope: Scope) -> Status {
    scope.borrow_mut().pop2()?;
    Ok(())
}

fn pop3(scope: Scope) -> Status {
    scope.borrow_mut().pop3()?;
    Ok(())
}

fn pop_dip(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let (_, top) = s.pop2()?;
    s.push(top);
    Ok(())
}

fn dup2(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let (left, right) = s.pop2()?;
    s.push_all([left.clone(), right.clone(), left, right]);
    Ok(())
}

fn dup3(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let (a, b, c) = s.pop3()?;
    s.push_all([a.clone(), b.clone(), c.clone(), a, b, c]);
    Ok(())
}

fn pick(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let (a, b, c) = s.pop3()?;
    s.push_all([a.clone(), b, c, a]);
    Ok(())
}

fn logical_and(_: Scope, left: Value, right: Value) -> Status<Value> {
    Ok(Value::from(left.as_bool()? && right.as_bool()?))
}

fn logical_or(_: Scope, left: Value, right: Value) -> Status<Value> {
    Ok(Value::from(left.as_bool()? || right.as_bool()?))
}

fn floor(_: Scope, value: Value) -> Status<Value> {
    if value.is_int() {
        Ok(value)
    } else {
        Ok(Value::from(value.as_float()?.floor()))
    }
}

fn ceil(_: Scope, value: Value) -> Status<Value> {
    if value.is_int() {
        Ok(value)
    } else {
        Ok(Value::from(value.as_float()?.ceil()))
    }
}

fn cast_to_int(_: Scope, value: Value) -> Status<Value> {
    Ok(Value::from(value.as_int()?))
}

fn cast_to_float(_: Scope, value: Value) -> Status<Value> {
    Ok(Value::from(value.as_float()?))
}

fn curry(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let (op, block) = s.pop2()?;
    s.push(StackVal::Function(Rc::new(CurriedFunc::new(
        op,
        block.as_fn()?,
    ))));
    Ok(())
}

fn bind(scope: Scope) -> Status {
    let mut s = scope.borrow_mut();
    let (block1, block2) = s.pop2()?;
    s.push(StackVal::Function(Rc::new(ComposedFunc::new(
        block1.as_fn()?,
        block2.as_fn()?,
    ))));
    Ok(())
}
