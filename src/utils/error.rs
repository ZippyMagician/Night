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
    Syntax(String),
    UnsupportedType(String),
    UndefinedSymbol(String),
    SymbolRedefinition(String),
    Unimplemented(String),
    // TODO: whatever else I need
}

macro_rules! night_err {
    ($type:ident, $msg:expr) => {
        Err(crate::utils::error::NightError::$type($msg.to_string()))
    };

    ($type:ident) => {
        Err(crate::utils::error::NightError::$type)
    };
}

pub(crate) use night_err;

pub type Status<T = ()> = Result<T, NightError>;

impl Display for NightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use NightError::*;

        match self {
            Pass => unreachable!(),
            Fail => write!(f, "Error: Program failed."),
            NothingToPop => write!(f, "StackError: Missing value to pop."),
            NaN => write!(f, "TypeError: Not a valid number"),
            Syntax(s) => write!(f, "SyntaxError: {s}"),
            UnsupportedType(s) => write!(f, "TypeError: {s}"),
            UndefinedSymbol(s) => write!(f, "UndefinedError: '{s}' is undefined."),
            SymbolRedefinition(s) => write!(f, "StackError: Attempted to redefine symbol '{s}'."),
            Unimplemented(s) => write!(f, "ImplementationError: '{s}' is unimplemented."),
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
