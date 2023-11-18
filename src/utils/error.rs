use std::error::Error;
use std::fmt::{self, Display};
use std::num::ParseIntError;

use crate::lexer::Span;

// Shorthand macro for calling crate::utils::error::error, used in `lexer.js`
macro_rules! lex_err {
    ($msg:expr ; $code:expr, $start:expr, $len:expr, $line_start:expr => $line_end:expr) => {
        crate::utils::error::error(
            $msg,
            crate::lexer::Span::span($code, $start, $len, $line_start, $line_end),
        )
    };
}

pub(crate) use lex_err;

pub fn error(msg: impl Display, span: Span<'_>) -> ! {
    println!("{msg} @ {span}");
    std::process::exit(-1)
}

#[derive(Clone, Debug)]
pub enum NightError {
    Pass,
    Fail,
    NothingToPop,
    NaN,
    UnsupportedType(String),
    SymbolRedefinition(String),
    Unimplemented(String),
    // TODO: whatever else I need
}

pub type Status<T = ()> = Result<T, NightError>;

impl Display for NightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use NightError::*;

        match self {
            Pass => unreachable!(),
            Fail => write!(f, "Error."),
            NothingToPop => write!(f, "StackError: Missing value to pop."),
            NaN => write!(f, "TypeError: Not a valid number"),
            UnsupportedType(s) => write!(f, "TypeError: {s}"),
            SymbolRedefinition(s) => write!(f, "StackError: Attempted to redefine symbol '{s}'."),
            Unimplemented(s) => write!(f, "InternalError: '{s}' is unimplemented."),
        }
    }
}

impl Error for NightError {}

impl From<ParseIntError> for NightError {
    fn from(_: ParseIntError) -> Self {
        NightError::NaN
    }
}

impl<T> From<NightError> for Status<T>
where
    T: Clone,
{
    fn from(value: NightError) -> Self {
        Err(value)
    }
}
