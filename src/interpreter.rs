use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::vec::IntoIter;

use crate::builtin::{Builtin, Operator, BUILTIN_MAP};
use crate::lexer::{LexTok, Span, Token};
use crate::scope::{Scope, ScopeInternal};
use crate::utils::error::{self, Status};
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
    StartArray(usize),
    EndArray(usize),
    StartParen(usize),
    EndParen(usize),
    Define(bool, usize),
}

pub struct Night<'a> {
    _code: Box<str>,
    tokens: IntoIter<LexTok<'a>>,
    spans: Vec<Span<'a>>,

    instrs: VecDeque<Instr>,
    scope: Scope,
}

macro_rules! push_instr {
    ($inst:expr, $arg:expr, $s:expr) => {
        $s.instrs.push_front($inst($arg, $s.spans.len() - 1))
    };

    ($inst:expr, $arg1:expr, $arg2:expr, $s:expr) => {
        $s.instrs
            .push_front($inst($arg1, $arg2, $s.spans.len() - 1))
    };
}

impl<'a> Night<'a> {
    pub fn init(code: &str, tokens: Vec<LexTok<'a>>) -> Self {
        Self {
            _code: code.into(),
            tokens: tokens.into_iter(),
            spans: vec![],
            instrs: VecDeque::new(),
            scope: Rc::new(RefCell::new(ScopeInternal::create())),
        }
    }

    pub fn get_scope(&self) -> Scope {
        self.scope.clone()
    }

    pub fn build_instrs(&mut self) {
        while let Some((tok, span)) = self.tokens.next() {
            self.spans.push(span.clone());
            if let Err(e) = self.build_instr(tok) {
                error::error(e, span);
            }
        }
    }

    fn build_instr(&mut self, tok: Token) -> Status {
        match tok {
            Token::Number(n) => push_instr!(Instr::Push, Value::from(n.parse::<i32>()?), self),
            Token::String(s) => push_instr!(Instr::Push, Value::from(s.to_string()), self),
            Token::Register(s) => push_instr!(Instr::PushSym, s.to_string(), true, self),
            Token::Op(o) => push_instr!(Instr::Op, o, self),
            Token::Symbol(_) => self.instrs.push_front(self.maybe_builtin(tok)),
            Token::Newline | Token::EOF => {} // skip
            _ => todo!(),
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

    pub fn exec_all(&mut self) {
        while let Some(instr) = self.instrs.pop_back() {
            self.exec(instr);
        }
    }

    #[inline]
    pub fn exec(&mut self, instr: Instr) {
        use Instr::*;

        match instr {
            Push(v, _) => self.scope.borrow_mut().push_val(v),
            Op(o, _) => {
                if let Err(e) = o.call(self.scope.clone()) {
                    eprintln!("Error(operator): {e:?}");
                }
            }
            Internal(b, _) => {
                if let Err(e) = b.call(self.scope.clone()) {
                    eprintln!("Error(builtin): {e:?}");
                }
            }
            _ => unimplemented!(),
        }
    }
}
