use crate::lexer::Span;

// Shorthand macro for calling crate::utils::error::error, used in `lexer.js`
macro_rules! lex_err {
    ($msg:expr ; $code:expr, $start:expr, $len:expr, $line_start:expr => $line_end:expr) => {{
        use crate::lexer::Span;
        use crate::utils::error::error;
        error(
            $msg,
            Span::span($code, $start, $len, $line_start, $line_end),
        )
    }};
}

pub(crate) use lex_err;

pub fn error(msg: &str, span: Span<'_>) -> ! {
    println!("Error: '{msg}' @ {span}");
    std::process::exit(-1)
}
