use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{self, Display};
use std::rc::Rc;
use std::vec::IntoIter;

use crate::builtin::{Builtin, Operator, BUILTIN_MAP};
use crate::lexer::{LexTok, Span, Token};
use crate::scope::{Scope, ScopeInternal};
use crate::utils::error::{self, night_err, NightError, Status};
use crate::utils::function::InlineFunction;
use crate::value::Value;

#[derive(Clone, Debug)]
pub enum Instr {
    Push(Value, usize),
    // PushFunc(...),
    PushSym(String, bool, usize),
    Op(Operator, usize),
    Internal(Builtin, usize),
    // Call(...),
    Guard(Vec<String>),
    Drop(Vec<String>),
    StartBlock(usize),
    EndBlock { start: usize, len: usize },
    StartArray(usize),
    EndArray { start: usize, len: usize },
    StartParen(usize),
    EndParen { start: usize, len: usize },
    Define(bool, usize),
}

pub struct Night<'a> {
    _code: Box<str>,
    tokens: IntoIter<LexTok<'a>>,
    spans: Vec<Span<'a>>,

    // It's easier to use a deque, since I can use a while `pop_back` and then easily modify in between iterations
    instrs: VecDeque<Instr>,
    scope: Scope,
}

macro_rules! push_instr {
    ($inst:expr, $s:expr) => {
        $s.instrs.push_front($inst($s.spans.len() - 1))
    };

    ($inst:expr, $arg:expr, $s:expr) => {
        $s.instrs.push_front($inst($arg, $s.spans.len() - 1))
    };

    ($inst:expr, $arg1:expr, $arg2:expr, $s:expr) => {
        $s.instrs
            .push_front($inst($arg1, $arg2, $s.spans.len() - 1))
    };
}

impl<'a> Night<'a> {
    pub fn new(code: &str, tokens: Vec<LexTok<'a>>) -> Self {
        Self {
            _code: code.into(),
            tokens: tokens.into_iter(),
            spans: vec![],
            instrs: VecDeque::new(),
            scope: Rc::new(RefCell::new(ScopeInternal::create())),
        }
    }

    pub fn child_process(&self, instrs: VecDeque<Instr>) -> Self {
        Self {
            _code: self._code.clone(),
            tokens: vec![].into_iter(),
            spans: self.spans.clone(),
            instrs,
            scope: Rc::new(RefCell::new(self.scope.borrow().to_owned().clone())),
        }
    }

    pub fn get_scope(&self) -> Scope {
        self.scope.clone()
    }

    pub fn init(&mut self) {
        while let Some((tok, span)) = self.tokens.next() {
            self.spans.push(span.clone());
            if let Err(e) = self.build_instr(tok) {
                error::error(e, span);
            }
        }
    }

    #[inline]
    fn build_instr(&mut self, tok: Token) -> Status {
        match tok {
            Token::Number(n) => push_instr!(Instr::Push, Value::from(n.parse::<i32>()?), self),
            Token::String(s) => push_instr!(Instr::Push, Value::from(s.to_string()), self),
            Token::Register(s) => push_instr!(Instr::PushSym, s.to_string(), true, self),
            Token::Op(o) => push_instr!(Instr::Op, o, self),
            Token::OpenCurly => self.parse_block()?,
            Token::CloseCurly => return night_err!(Syntax, "Unbalanced block."),
            Token::Define => self.parse_define()?,
            Token::Symbol(_) => self.instrs.push_front(self.maybe_builtin(tok)),
            Token::Newline | Token::EOF => {} // skip
            Token::Pipe => return night_err!(Syntax, "Invalid usage of the 'Const' identifier."),
            _ => return night_err!(Unimplemented, format!("Token '{tok:?}'.")),
        }

        Ok(())
    }

    fn maybe_builtin(&self, tok: Token) -> Instr {
        if let Token::Symbol(s) = tok {
            if let Some(&b) = BUILTIN_MAP.get(s) {
                Instr::Internal(b, self.spans.len() - 1)
            } else if let Some(o) = Operator::from_name(s) {
                Instr::Op(o, self.spans.len() - 1)
            } else {
                Instr::PushSym(s.to_string(), false, self.spans.len() - 1)
            }
        } else {
            unreachable!()
        }
    }

    fn parse_block(&mut self) -> Status {
        let mut span_queue = Vec::new();
        span_queue.push(self.spans.len() - 1);
        push_instr!(Instr::StartBlock, self);

        while let Some((t, s)) = self.tokens.next() {
            self.spans.push(s);

            // Avoid excessive recursion. Technically I could just call `self.build_instr` without the if statement
            if matches!(t, Token::CloseCurly) {
                // Will never encounter a situation where it is unbalanced inside
                let start = span_queue.pop().unwrap();
                self.instrs.push_front(Instr::EndBlock {
                    start,
                    len: self.spans.len() - start - 2,
                });
                if span_queue.is_empty() {
                    break;
                }
            } else if matches!(t, Token::OpenCurly) {
                span_queue.push(self.spans.len() - 1);
                push_instr!(Instr::StartBlock, self);
            } else {
                self.build_instr(t)?;
            }
        }

        Ok(())
    }

    fn parse_define(&mut self) -> Status {
        let def_span = self.spans.len() - 1;
        let name = match self.tokens.next() {
            Some((Token::Symbol(s), _)) => s,
            _ => return night_err!(Syntax, "Expected a 'Symbol' to follow the 'Define' declaration."),
        };
        let sym = Instr::Push(Value::from(name), 0);

        let mut def = VecDeque::new();
        let (start, span) = self.tokens.next().ok_or(NightError::Syntax(
            "Definition cannot be empty.".to_string(),
        ))?;
        let is_const = matches!(start, Token::Pipe);
        let orig_len = self.instrs.len();

        if !is_const {
            self.spans.push(span);
            self.build_instr(start)?;
        }

        while let Some((tok, s)) = self.tokens.next() {
            self.spans.push(s);
            if tok == Token::Newline || tok == Token::EOF {
                break;
            }
            self.build_instr(tok)?;
        }

        let len = self.instrs.len() - orig_len;
        for _ in 0..len {
            def.push_back(self.instrs.pop_front().unwrap());
        }

        if is_const {
            let mut child = self.child_process(def);
            child.exec();
            let mut scope = child.get_scope().borrow().to_owned();
            if scope.stack_len() != 1 {
                return Err(NightError::NothingToPop);
            }

            self.instrs
                .push_front(Instr::Push(scope.pop_value()?, def_span));
        } else {
            self.instrs.push_front(Instr::StartBlock(def_span));
            for instr in def.into_iter().rev() {
                self.instrs.push_front(instr);
            }
            self.instrs.push_front(Instr::EndBlock {
                start: def_span,
                len,
            });
        }

        self.instrs.push_front(sym);
        self.instrs
            .push_front(Instr::Internal(Builtin::Def, def_span));

        Ok(())
    }

    pub fn exec(&mut self) {
        while let Some(instr) = self.instrs.pop_back() {
            self.exec_instr(instr);
        }
    }

    #[inline]
    pub fn exec_instr(&mut self, instr: Instr) {
        use Instr::*;

        match instr {
            Push(v, _) => self.scope.borrow_mut().push_value(v),
            PushSym(v, false, i) => {
                let mut s = self.scope.borrow_mut();
                let value = match s.get_def(v) {
                    Ok(inner) => inner.clone(),
                    Err(e) => error::error(e, self.spans[i].clone()),
                };
                s.push(value);
            }
            Op(o, i) => {
                if let Err(e) = o.call(self.scope.clone()) {
                    error::error(e, self.spans[i].clone());
                }
            }
            Internal(b, i) => {
                if let Err(e) = b.call(self.scope.clone()) {
                    error::error(e, self.spans[i].clone());
                }
            }
            _ => todo!(),
        }
    }
}

impl<'a> Display for Night<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Night")
            .field("instrs", &self.instrs)
            .field("stack", &format!("{}", self.scope.borrow()))
            .finish()
    }
}
