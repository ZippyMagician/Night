use std::cell::RefCell;
use std::rc::Rc;

use crate::builtin::{Builtin, Operator, BUILTIN_MAP};
use crate::lexer::{LexTok, Span, Token};
use crate::scope::{Scope, ScopeInternal};
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
    _orig_span: Box<str>,
    _instrs: Vec<Instr>,
    _spans: Vec<Span<'a>>,
    scope: Scope,
}

impl<'a> Night<'a> {
    pub fn init(code: &str, instrs: Vec<Instr>) -> Self {
        Self {
            _orig_span: code.into(),
            _instrs: instrs,
            _spans: vec![],
            scope: Rc::new(RefCell::new(ScopeInternal::create())),
        }
    }

    // TODO: Temporarily pub so I can test a few things
    pub fn maybe_builtin(&mut self, tok: LexTok<'a>) -> Instr {
        match tok.0 {
            Token::Symbol(s) => {
                self._spans.push(tok.1);
                if let Some(&b) = BUILTIN_MAP.get(s) {
                    Instr::Internal(b, self._spans.len() - 1)
                } else if let Some(o) = Operator::from_name(s) {
                    Instr::Op(o, self._spans.len() - 1)
                } else {
                    Instr::PushSym(s.to_string(), false, self._spans.len() - 1)
                }
            }
            _ => unreachable!(),
        }
    }

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
