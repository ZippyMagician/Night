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

pub fn error(msg: &str, span: Span<'_>) -> ! {
    println!("Error: '{msg}' @ {span}");
    std::process::exit(-1)
}

#[derive(Clone, Copy, Debug)]
pub enum NightError {
    Pass,
    MissingArg,
    InvalidParam,
    // TODO
}

#[derive(Clone, Debug)]
pub struct Status<T> {
    pub data: T,
    pub info: NightError,
}

impl<T> Status<T> {
    pub fn pass(data: T) -> Self {
        Self {
            data,
            info: NightError::Pass,
        }
    }
}
