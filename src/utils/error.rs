use std::error::Error;
use std::fmt::{self, Display};
use std::num::{ParseFloatError, ParseIntError};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Span {
    code: Rc<str>,
    start: usize,
    len: usize,
    line_start: usize,
    line_end: usize,
}

impl Span {
    pub fn empty() -> Self {
        Self {
            code: "".into(),
            start: 0,
            len: 0,
            line_start: 0,
            line_end: 0,
        }
    }

    pub fn span(
        code: Rc<str>,
        start: usize,
        len: usize,
        line_start: usize,
        line_end: usize,
    ) -> Self {
        Self {
            code,
            start,
            len,
            line_start,
            line_end,
        }
    }

    pub fn between(left: &Span, right: &Span) -> Self {
        Self {
            code: left.code.clone(),
            start: left.start,
            len: right.start.abs_diff(left.start + left.len) + left.len + right.len,
            line_start: std::cmp::min(left.line_start, right.line_start),
            line_end: std::cmp::max(left.line_end, right.line_end),
        }
    }

    pub fn as_lit(&self) -> &[u8] {
        &self.code.as_bytes()[self.start..self.start + self.len]
    }

    fn fmt_line(&self, line: usize) -> String {
        format!("{line:>7}| {}", self.code.lines().nth(line).unwrap().trim())
    }

    fn fmt_arrow(&self, on_start: bool, start: usize, len: usize) -> String {
        let l = self
            .code
            .lines()
            .nth(if on_start {
                self.line_start
            } else {
                self.line_end
            })
            .unwrap();
        let diff = l.len() - l.trim_start().len();

        let mut buf = String::with_capacity(start + len - diff + 9);
        buf.push_str("         ");
        for i in 0..(start + len - diff) {
            if i < start - diff {
                buf.push(' ');
            } else {
                buf.push('_');
            }
        }
        format!("{buf} <--")
    }

    fn get_index(&self) -> (usize, usize) {
        let offset: usize = self.code.lines().take(self.line_start).map(str::len).sum();
        let left = self.start - offset - self.line_start;

        let right;
        if self.line_start == self.line_end {
            right = left + self.len;
        } else {
            let offset: usize = self.code.lines().take(self.line_end).map(str::len).sum();
            right = self.start + self.len - offset - self.line_end;
        }

        (left, right)
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lefti, righti) = self.get_index();
        writeln!(
            f,
            "[({}:{}) => ({}:{})]:",
            self.line_start, lefti, self.line_end, righti
        )?;

        if self.line_start == self.line_end {
            writeln!(f, "{}", self.fmt_line(self.line_start))?;
            writeln!(f, "{} Here.", self.fmt_arrow(true, lefti, self.len))
        } else {
            writeln!(f, "{}", self.fmt_line(self.line_start))?;
            writeln!(f, "{} From here...", self.fmt_arrow(true, lefti, 1))?;
            writeln!(f, "{}", self.fmt_line(self.line_end))?;
            writeln!(f, "{} ...to here.", self.fmt_arrow(false, righti, 1))
        }
    }
}

// Shorthand macro for calling crate::utils::error::error, used in `lexer.js`
macro_rules! lex_err {
    ($msg:expr ; $code:expr, $start:expr, $len:expr, $line_start:expr => $line_end:expr) => {
        crate::utils::error::error(
            $msg,
            crate::utils::error::Span::span($code.clone(), $start, $len, $line_start, $line_end),
        )
    };
}

pub(crate) use lex_err;

pub fn error(msg: impl Display, span: Span) -> ! {
    println!("{msg} {span}");
    std::process::exit(-1)
}

pub fn warn(msg: impl Display, span: Span) {
    println!("Warning: {msg} {span}")
}

pub fn error_with_trace(msg: impl Display, span: Span, trace: Vec<Span>) -> ! {
    println!("{msg} {span}");
    for s in trace {
        print!("Called from {s}");
    }

    std::process::exit(-1);
}

#[derive(Clone, Debug)]
pub enum NightError {
    Pass,
    Fail,
    NothingToPop,
    NaN,
    ContextFail(String),
    Syntax(String),
    UnsupportedType(String),
    UndefinedSymbol(String),
    SymbolRedefinition(String),
    Unimplemented(String),
    Runtime(String),
    Warning(String),
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
            ContextFail(s) => write!(f, "Error: {s}"),
            NothingToPop => write!(f, "StackError: Missing value to pop."),
            NaN => write!(f, "TypeError: Not a valid number."),
            Syntax(s) => write!(f, "SyntaxError: {s}"),
            UnsupportedType(s) => write!(f, "TypeError: {s}"),
            UndefinedSymbol(s) => write!(f, "UndefinedError: '{s}' is undefined."),
            SymbolRedefinition(s) => write!(f, "StackError: Attempted to redefine symbol '{s}'."),
            Unimplemented(s) => write!(f, "ImplementationError: '{s}' is unimplemented."),
            Runtime(s) => write!(f, "RuntimeError: {s}"),
            Warning(s) => write!(f, "Warning: {s}"),
        }
    }
}

impl Error for NightError {}

impl From<ParseIntError> for NightError {
    fn from(_: ParseIntError) -> Self {
        NightError::NaN
    }
}

impl From<ParseFloatError> for NightError {
    fn from(_: ParseFloatError) -> Self {
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
