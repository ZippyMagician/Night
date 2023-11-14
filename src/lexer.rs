use std::fmt::{self, Display};
use std::iter::Peekable;
use std::str::CharIndices;

use crate::utils::error::lex_err;
use crate::utils::token::{Token, OP_MAP};

// Struct representing where in the input string some `Token` actually is represented
#[derive(Clone, Debug)]
pub struct Span<'a> {
    code: &'a str,
    start: usize,
    len: usize,
    line_start: usize,
    line_end: usize,
}

impl<'a> Span<'a> {
    fn span(code: &'a str, start: usize, len: usize, line_start: usize, line_end: usize) -> Self {
        Self {
            code,
            start,
            len,
            line_start,
            line_end,
        }
    }

    fn fmt_line(&self, line: usize) -> String {
        format!("{line:>7}| {}\n", self.code.split('\n').nth(line).unwrap())
    }

    fn fmt_arrow(&self, start: usize, len: usize) -> String {
        let mut buf = String::with_capacity(start + len);
        for i in 0..(start + len) {
            if i < start {
                buf.push(' ');
            } else {
                buf.push('_');
            }
        }
        format!("{buf} <--")
    }

    fn get_index(&self) -> (usize, usize) {
        let left = self.start
            - self
                .code
                .split('\n')
                .take(self.line_start)
                .map(str::len)
                .sum::<usize>()
            - self.line_start;
        let right;
        if self.line_start == self.line_end {
            right = left + self.len - 1;
        } else {
            let offset: usize = self
                .code
                .split('\n')
                .skip(self.line_start)
                .take(self.line_end - self.line_start)
                .map(str::len)
                .sum();
            right = left + self.len - offset - self.line_end + self.line_start;
        }
        (left, right)
    }
}

impl<'a> Display for Span<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lefti, righti) = self.get_index();
        writeln!(
            f,
            "[({}:{}) => ({}:{})]:",
            self.line_start, lefti, self.line_end, righti
        )?;
        if self.line_start == self.line_end {
            write!(f, "{}", self.fmt_line(self.line_start))?;
            writeln!(f, "         {} Here.", self.fmt_arrow(lefti, self.len))?;
        } else {
            write!(f, "{}", self.fmt_line(self.line_start))?;
            writeln!(f, "         {} From here...", self.fmt_arrow(lefti, 1))?;
            write!(f, "{}", self.fmt_line(self.line_end))?;
            writeln!(f, "         {} ...to here.", self.fmt_arrow(righti, 1))?;
        }
        Ok(())
    }
}

// Type alias for the type used by the `Lexer`
pub type LexTok<'a> = (Token, Span<'a>);

// Container struct that consumes a string to create a list of tokens
pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
}

// Shorthand for writing out (tok, Span::span(/* ... */))
macro_rules! lex_tok {
    ($t:expr, $s:ident, $start:expr, $len:expr, $lines:expr) => {{
        let span = crate::lexer::Span::span;
        ($t, span($s.input, $start, $len, $s.line, $s.line + $lines))
    }};
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            line: 0,
        }
    }

    #[inline]
    fn is_peek_digit(&mut self) -> bool {
        self.chars.peek().map_or(false, |(_, c)| c.is_ascii_digit())
    }

    #[inline]
    fn is_peek_var(&mut self) -> bool {
        self.chars.peek().map_or(false, |&(_, c)| match c {
            'a'..='z' => true,
            'A'..='Z' => true,
            '_' => true,
            '0'..='9' => true,
            _ => false,
        })
    }

    // Main function, run to fully tokenize an input string
    pub fn tokenize(&mut self) -> Vec<LexTok<'a>> {
        let mut tokens = Vec::new();
        while let Some(tok) = self.consume_token() {
            tokens.push(tok);
        }
        tokens
    }

    fn maybe_op(&mut self, chr: char, start: usize) -> Option<LexTok<'a>> {
        match chr {
            '-' if self.is_peek_digit() => self.consume_number(chr, start),
            ':' if self.is_peek_var() => self.consume_word(chr, start),
            '[' => Some(lex_tok!(Token::OpenBracket, self, start, 1, 0)),
            ']' => Some(lex_tok!(Token::CloseBracket, self, start, 1, 0)),
            '{' => Some(lex_tok!(Token::OpenCurly, self, start, 1, 0)),
            '}' => Some(lex_tok!(Token::CloseCurly, self, start, 1, 0)),
            _ => {
                if OP_MAP.contains_key(&chr.to_string()) {
                    let mut buf = chr.to_string();
                    while let Some(&(_, chr)) = self.chars.peek() {
                        buf.push(chr);
                        if OP_MAP.contains_key(&buf) {
                            self.chars.next();
                        } else {
                            buf.pop();
                            break;
                        }
                    }
                    let (_, &op) = OP_MAP.get_entry(&buf).unwrap();
                    Some(lex_tok!(Token::Operator(op), self, start, buf.len(), 0))
                } else {
                    lex_err!("Unrecognized token."; self.input, start, 1, self.line => self.line);
                    None
                }
            }
        }
    }

    #[inline]
    fn consume_token(&mut self) -> Option<LexTok<'a>> {
        let (start, chr) = self.chars.next()?;
        match chr {
            '0'..='9' => self.consume_number(chr, start),
            '$' => self.consume_word(chr, start),
            '"' => self.consume_string(start),
            'a'..='z' | 'A'..='Z' => self.consume_word(chr, start),
            '\n' => {
                let tok = lex_tok!(Token::Newline, self, start, 1, 0);
                self.line += 1;
                Some(tok)
            }
            ' ' | '\t' => self.consume_token(),
            c if c.is_ascii_punctuation() => self.maybe_op(chr, start),
            _ => {
                lex_err!("Unrecognized token."; self.input, start, 1, self.line => self.line);
                None
            }
        }
    }

    fn consume_number(&mut self, buf: char, start: usize) -> Option<LexTok<'a>> {
        let mut buf = buf.to_string();
        while self.is_peek_digit() {
            buf.push(self.chars.next()?.1);
            self.chars.next();
        }
        if let Ok(n) = buf.parse() {
            Some(lex_tok!(Token::Number(n), self, start, buf.len(), 0))
        } else {
            None
        }
    }

    fn consume_word(&mut self, chr: char, start: usize) -> Option<LexTok<'a>> {
        let mut buf;
        let offset;
        if matches!(chr, '$' | ':') {
            buf = String::new();
            offset = 1;
        } else {
            buf = chr.to_string();
            offset = 0;
        }

        while self.is_peek_var() {
            buf.push(self.chars.next()?.1);
            self.chars.next();
        }

        if buf.is_empty() {
            None
        } else {
            let len = buf.len();
            Some(lex_tok!(
                match chr {
                    '$' => Token::Register(buf),
                    ':' => Token::Word(buf),
                    _ => Token::Variable(buf),
                },
                self,
                start,
                len + offset,
                0
            ))
        }
    }

    fn consume_string(&mut self, start: usize) -> Option<LexTok<'a>> {
        let mut buf = String::new();
        let mut valid_str = false;
        let mut lines = 0;
        while let Some((_, chr)) = self.chars.next() {
            if chr == '"' {
                valid_str = true;
                break;
            } else if chr == '\n' {
                lines += 1;
            }
            buf.push(chr);
        }

        self.line += lines;
        if valid_str {
            let len = buf.len();
            let tok = lex_tok!(Token::String(buf), self, start, len + 2, lines);
            self.line += lines;
            Some(tok)
        } else {
            lex_err!("String not terminated."; self.input, start, buf.len(), self.line - lines => self.line);
            None
        }
    }
}
