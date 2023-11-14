use phf::phf_map;

use super::Operator;

macro_rules! define {
    ($($rep:expr => ($lit:pat));*) => {
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

define! {
    "+" => (Operator::Add);

    "-" => (Operator::Sub);

    "/" => (Operator::Div);

    "*" => (Operator::Mul);

    "=" => (Operator::Eq);

    "!=" => (Operator::Neq);

    ">" => (Operator::Gt);

    "<" => (Operator::Lt);

    ">=" => (Operator::Gte);

    "<=" => (Operator::Lte);

    "!" => (Operator::Assign);

    ";" => (Operator::Pop);

    ":" => (Operator::Swap);

    "." => (Operator::Dup);

    "?" => (Operator::Call)
}
