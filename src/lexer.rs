use std::iter::Peekable;
use std::str::CharIndices;

use crate::builtin::{Operator, OP_MAP};
use crate::utils;
use crate::utils::error::{lex_err, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
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
    fn next_if(&mut self, f: impl FnOnce(char) -> bool) -> Option<(usize, char)> {
        self.chars.next_if(|&(_, c)| f(c))
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
        match chr {
            '0'..='9' => self.consume_number(start),
            'a'..='z' | 'A'..='Z' => self.consume_symbol(start),
            c if c.is_whitespace() => self.consume_whitespace(c, start),
            c if c.is_ascii_punctuation() => self.maybe_op(c, start),
            _ => {
                lex_err!("LexError: Unrecognized token."; self.input, start, 1, self.line => self.line)
            }
        }
    }

    fn maybe_op(&mut self, chr: char, start: usize) -> Option<LexTok<'a>> {
        if chr == '-' && self.next_if(|c| c == '-').is_some() {
            return self.skip_comment();
        } else if chr == '-' && self.next_if(|c| c == '>').is_some() {
            return lex_tok!(Token::Define, self, start, 2, 0);
        } else if chr == '-' && self.next_if(|c| c.is_ascii_digit()).is_some() {
            return self.consume_number(start);
        // This uses `peek` instead of `next_if` in order to avoid issues with the 1st char of the word being consumed before `calculate_var_bounds` is called.
        } else if chr == ':'
            && self
                .chars
                .peek()
                .map_or(false, |&(_, c)| utils::valid_symbol_chr(c))
        {
            return self.consume_word(start);
        }

        match chr {
            '\'' => self.consume_char_lit(start),
            '$' => self.consume_register(start),
            '"' => self.consume_string(start),
            '|' => lex_tok!(Token::Pipe, self, start, 1, 0),
            '[' => lex_tok!(Token::OpenBracket, self, start, 1, 0),
            ']' => lex_tok!(Token::CloseBracket, self, start, 1, 0),
            '{' => lex_tok!(Token::OpenCurly, self, start, 1, 0),
            '}' => lex_tok!(Token::CloseCurly, self, start, 1, 0),
            '(' => lex_tok!(Token::OpenParen, self, start, 1, 0),
            ')' => lex_tok!(Token::CloseParen, self, start, 1, 0),
            _ if OP_MAP.contains_key(&self.input[start..start + 1]) => self.consume_op(start),
            _ => {
                lex_err!("LexError: Unrecognized token."; self.input, start, 1, self.line => self.line)
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
        let mut found_decimal = false;
        while let Some((_, c)) = self.next_if(|c| c.is_ascii_digit() || c == '.' && !found_decimal)
        {
            if c == '.' {
                found_decimal = true;
            }
            end += 1;
        }
        lex_tok!(Token::Number, start, end, self, start, end - start, 0)
    }

    fn calculate_var_bounds(&mut self, start: usize) -> (usize, usize) {
        let mut end = start + 1;
        while self.next_if(utils::valid_symbol_chr).is_some() {
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
            lex_err!("LexError: Missing identifier for register."; self.input, start, 1, self.line => self.line);
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
            lex_err!("LexError: String not terminated."; self.input, start, span, self.line => self.line + lines);
        }

        let tok = lex_tok!(Token::String, start + 1, end - 1, self, start, span, lines);
        self.line += lines;
        tok
    }

    fn consume_op(&mut self, start: usize) -> Option<LexTok<'a>> {
        let mut end = start + 1;
        while self
            .next_if(|_| OP_MAP.contains_key(&self.input[start..end + 1]))
            .is_some()
        {
            end += 1;
        }

        let (_, &op) = OP_MAP.get_entry(&self.input[start..end])?;
        lex_tok!(Token::Op(op), self, start, end - start, 0)
    }

    fn consume_char_lit(&mut self, start: usize) -> Option<LexTok<'a>> {
        if self.chars.next().is_none() {
            lex_err!("LexError: Missing following char identifier."; self.input, start, 1, self.line => self.line);
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
