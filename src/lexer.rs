use std::fmt::{self, Display};
use std::iter::Peekable;
use std::str::CharIndices;

use crate::builtin::{Operator, OP_MAP};
use crate::utils::error::lex_err;

#[derive(Clone, Debug)]
pub enum Token<'a> {
    /// `15`, `-3`
    Number(&'a str),
    /// `$x`, `$_for_i`
    Register(&'a str),
    /// `"hello world"`, `:goodbye`, `"hi"`, `'a`
    String(&'a str),
    /// `for`, `print`, `add`, etc.
    Symbol(&'a str),
    /// `+`, `!=`, `.`, etc
    Op(Operator),
    /// Has semantic meaning, unlike other whitespace
    /// e.g. in `-> x : . + \n 1 2 3 x`, x's definition should end with the newline
    Newline,
    /// EOF
    EOF,
    /// `->`
    Define,
    /// `|`
    Pipe,
    /// `{`
    OpenCurly,
    /// `}`
    CloseCurly,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
}

#[derive(Clone, Debug)]
pub struct Span<'a> {
    code: &'a str,
    start: usize,
    len: usize,
    line_start: usize,
    line_end: usize,
}

impl<'a> Span<'a> {
    pub fn empty() -> Self {
        Self {
            code: "",
            start: 0,
            len: 0,
            line_start: 0,
            line_end: 0,
        }
    }

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
        format!("{line:>7}| {}\n", self.code.lines().nth(line).unwrap())
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
                .lines()
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
                .lines()
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

pub type LexTok<'a> = (Token<'a>, Span<'a>);

pub struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
    line: usize,
}

/// Shorthand for writing out Some((tok, Span::span(/* ... */)))
macro_rules! lex_tok {
    ($t:expr, $s_start:expr, $s_end:expr, $s:expr, $start:expr, $len:expr, $lines:expr) => {{
        let span = crate::lexer::Span::span;
        let buf = &$s.input[$s_start..$s_end];
        Some((
            $t(buf),
            span($s.input, $start, $len, $s.line, $s.line + $lines),
        ))
    }};

    ($t:expr, $s:ident, $start:expr, $len:expr, $lines:expr) => {{
        let span = crate::lexer::Span::span;
        Some(($t, span($s.input, $start, $len, $s.line, $s.line + $lines)))
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
    fn is_peek_match(&mut self, f: fn(&char) -> bool) -> bool {
        self.chars.peek().map_or(false, |(_, c)| f(c))
    }

    /// Entry function for tokenization
    pub fn tokenize(&mut self) -> Vec<LexTok<'a>> {
        let mut tokens = Vec::new();
        while let Some(tok) = self.consume_token() {
            tokens.push(tok);
        }

        tokens.push((
            Token::EOF,
            Span::span(self.input, self.input.len() - 1, 1, self.line, self.line),
        ));
        tokens
    }

    #[inline]
    fn consume_token(&mut self) -> Option<LexTok<'a>> {
        let (start, chr) = self.chars.next()?;
        if chr.is_whitespace() {
            return self.consume_whitespace(chr, start);
        } else if chr.is_ascii_punctuation() {
            return self.maybe_op(chr, start);
        }

        match chr {
            '0'..='9' => self.consume_number(start),
            'a'..='z' | 'A'..='Z' => self.consume_symbol(start),
            _ => lex_err!("Unrecognized token."; self.input, start, 1, self.line => self.line),
        }
    }

    fn maybe_op(&mut self, chr: char, start: usize) -> Option<LexTok<'a>> {
        let peek = self.chars.peek().map(|&(_, c)| c);
        match (chr, peek) {
            ('\'', _) => self.consume_char_lit(start),
            ('-', Some('-')) => self.skip_comment(),
            ('-', Some('>')) => {
                self.chars.next();
                lex_tok!(Token::Define, self, start, 2, 0)
            }
            ('-', Some(c)) if c.is_ascii_digit() => self.consume_number(start),
            (':', Some(c @ ('-' | _))) if c.is_ascii_alphanumeric() => self.consume_word(start),
            ('$', _) => self.consume_register(start),
            ('"', _) => self.consume_string(start),
            ('|', _) => lex_tok!(Token::Pipe, self, start, 1, 0),
            ('[', _) => lex_tok!(Token::OpenBracket, self, start, 1, 0),
            (']', _) => lex_tok!(Token::CloseBracket, self, start, 1, 0),
            ('{', _) => lex_tok!(Token::OpenCurly, self, start, 1, 0),
            ('}', _) => lex_tok!(Token::CloseCurly, self, start, 1, 0),
            _ if OP_MAP.contains_key(&self.input[start..start + 1]) => self.consume_op(start),
            _ => {
                lex_err!("Unrecognized token."; self.input, start, 1, self.line => self.line)
            }
        }
    }

    fn consume_whitespace(&mut self, chr: char, start: usize) -> Option<LexTok<'a>> {
        if chr == '\n' {
            let tok = lex_tok!(Token::Newline, self, start, 1, 0);
            self.line += 1;
            return tok;
        }

        self.consume_token()
    }

    fn consume_number(&mut self, start: usize) -> Option<LexTok<'a>> {
        let mut end = start + 1;
        while self.is_peek_match(char::is_ascii_digit) {
            self.chars.next();
            end += 1;
        }
        lex_tok!(Token::Number, start, end, self, start, end - start, 0)
    }

    fn calculate_var_bounds(&mut self, start: usize) -> (usize, usize) {
        let mut end = start + 1;
        while self.is_peek_match(|&c| c == '-' || c.is_ascii_alphanumeric()) {
            self.chars.next();
            end += 1;
        }

        (start, end)
    }

    // The pass to convert matching symbols to built ins and operators occurs prior to execution
    fn consume_symbol(&mut self, start: usize) -> Option<LexTok<'a>> {
        let (start, end) = self.calculate_var_bounds(start);
        lex_tok!(Token::Symbol, start, end, self, start, end - start, 0)
    }

    fn consume_register(&mut self, start: usize) -> Option<LexTok<'a>> {
        let (start, end) = self.calculate_var_bounds(start);
        if end - start == 1 {
            lex_err!("Missing identifier for register."; self.input, start, 1, self.line => self.line);
        }
        lex_tok!(Token::Register, start + 1, end, self, start, end - start, 0)
    }

    fn consume_word(&mut self, start: usize) -> Option<LexTok<'a>> {
        let (start, end) = self.calculate_var_bounds(start);
        lex_tok!(Token::String, start + 1, end, self, start, end - start, 0)
    }

    fn consume_string(&mut self, start: usize) -> Option<LexTok<'a>> {
        let mut valid_str = false;
        let mut lines = 0;
        let mut end = start + 2;
        while let Some((_, chr)) = self.chars.next() {
            if chr == '"' {
                valid_str = true;
                break;
            } else if chr == '\n' {
                lines += 1;
            }
            end += 1;
        }

        let span = end - start;

        if !valid_str {
            lex_err!("String not terminated."; self.input, start, span, self.line => self.line + lines);
        }

        let tok = lex_tok!(Token::String, start + 1, end - 1, self, start, span, lines);
        self.line += lines;
        tok
    }

    fn consume_op(&mut self, start: usize) -> Option<LexTok<'a>> {
        let mut end = start + 1;
        while self.chars.peek().is_some() {
            if OP_MAP.contains_key(&self.input[start..end + 1]) {
                end += 1;
                continue;
            }

            break;
        }
        let (_, &op) = OP_MAP.get_entry(&self.input[start..end])?;
        lex_tok!(Token::Op(op), self, start, end - start, 0)
    }

    fn consume_char_lit(&mut self, start: usize) -> Option<LexTok<'a>> {
        if self.chars.next().is_none() {
            lex_err!("Missing following char identifier."; self.input, start, 1, self.line => self.line);
        }

        lex_tok!(Token::String, start + 1, start + 2, self, start, 1, 0)
    }

    fn skip_comment(&mut self) -> Option<LexTok<'a>> {
        while let Some((i, tok)) = self.chars.next() {
            if tok == '\n' {
                let t = lex_tok!(Token::Newline, self, i, 1, 0);
                self.line += 1;
                return t;
            }
        }

        None // End of file
    }
}
