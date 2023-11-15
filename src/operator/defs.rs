use phf::phf_map;

use super::Operator;

macro_rules! define {
    ($($rep:expr => ($lit:pat, $guard:expr));*) => {
        pub static OP_MAP: phf::Map<&'static str, crate::operator::Operator> = phf_map! {
            $(
               $rep => $lit
            ),*
        };

        impl crate::operator::Operator {
            pub fn get_glyph(&self) -> &'static str {
                match *self {
                    $(
                        $lit => $rep
                    ),*
                }
            }
        }
    }
}

// Operator mappings for the tokens and their literal repr
define! {
    "+" => (Operator::Add, None);

    "-" => (Operator::Sub, None);

    "/" => (Operator::Div, None);

    "*" => (Operator::Mul, None);

    "=" => (Operator::Eq, None);

    "!=" => (Operator::Neq, None);

    ">" => (Operator::Gt, None);

    "<" => (Operator::Lt, None);

    ">=" => (Operator::Gte, None);

    "<=" => (Operator::Lte, None);

    "!" => (Operator::Assign, None);

    ";" => (Operator::Pop, None);

    ":" => (Operator::Swap, None);

    "." => (Operator::Dup, None);

    "?" => (Operator::Call, None)
}
