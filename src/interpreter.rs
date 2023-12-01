use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{self, Debug, Display};
use std::rc::Rc;
use std::vec::IntoIter;

use crate::builtin::{Builtin, Operator, BUILTIN_MAP};
use crate::lexer::{LexTok, Span, Token};
use crate::scope::{Scope, ScopeInternal, StackVal};
use crate::utils;
use crate::utils::error::{self, night_err, NightError, Status};
use crate::utils::function::{BiFunction, InlineFunction};
use crate::value::Value;

#[derive(Clone)]
pub enum Instr {
    Push(Value, usize),
    // TODO: If I want to simplify some definitions, I need to change this to a `Rc<dyn InlineFunction>`
    // and figure out how to handle `BiFunction` for that case, so I can "unwrap" some blocks if they are
    // a single operator, builtin, or some symbol
    PushFunc(BiFunction, usize),
    PushSym(String, bool, usize),
    Op(Operator, usize),
    Internal(Builtin, usize),
    // Call(...),
    Guard(Vec<String>, usize),
    GuardEnd(Vec<String>, usize),
    //StartBlock(usize),
    //EndBlock { start: usize, len: usize },
    //StartArray(usize),
    //EndArray { start: usize, len: usize },
    //StartParen(usize),
    //EndParen { start: usize, len: usize },
}

impl Instr {
    pub fn get_span(&self) -> usize {
        match self {
            Instr::Push(_, s) => *s,
            Instr::PushFunc(_, s) => *s,
            Instr::PushSym(_, _, s) => *s,
            Instr::Op(_, s) => *s,
            Instr::Internal(_, s) => *s,
            Instr::Guard(_, s) => *s,
            Instr::GuardEnd(_, s) => *s,
        }
    }
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
        $s.instrs.push_back($inst($s.spans.len() - 1))
    };

    ($inst:expr, $arg:expr, $s:expr) => {
        $s.instrs.push_back($inst($arg, $s.spans.len() - 1))
    };

    ($inst:expr, $arg1:expr, $arg2:expr, $s:expr) => {
        $s.instrs.push_back($inst($arg1, $arg2, $s.spans.len() - 1))
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

    pub fn partial_new(instrs: impl Into<VecDeque<Instr>>, scope: Scope) -> Self {
        Self {
            _code: "".into(),
            tokens: vec![].into_iter(),
            spans: Vec::new(),
            instrs: instrs.into(),
            scope,
        }
    }

    pub fn clone_child(&self, instrs: impl Into<VecDeque<Instr>>) -> Self {
        Self {
            _code: self._code.clone(),
            tokens: vec![].into_iter(),
            spans: self.spans.clone(),
            instrs: instrs.into(),
            scope: Rc::new(RefCell::new(self.scope.borrow().to_owned().clone())),
        }
    }

    pub fn get_scope(&self) -> Scope {
        self.scope.clone()
    }

    pub fn init(&mut self) {
        while let Some((tok, span)) = self.tokens.next() {
            self.spans.push(span);
            if let Err(e) = self.build_instr(tok) {
                error::error(e, self.spans[self.spans.len() - 1].clone());
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
            Token::OpenParen => {
                let guard = self.parse_guard()?;
                match self.tokens.next() {
                    Some((Token::OpenCurly, span)) => {
                        self.spans.push(span);
                        self.parse_block(Some(guard))?;
                    }
                    _ => return night_err!(Syntax, "A guard statement is only valid either before the body of a '->' definition or preceeding a block."),
                }
            }
            Token::CloseParen => return night_err!(Syntax, "Unbalanced parenthesis."),
            Token::OpenCurly => self.parse_block(None)?,
            Token::CloseCurly => return night_err!(Syntax, "Unbalanced block."),
            Token::Define => self.parse_define()?,
            Token::Symbol(_) => self.instrs.push_back(self.maybe_builtin(tok)),
            Token::Newline | Token::EOF => {} // skip
            Token::Pipe => return night_err!(Syntax, "Invalid usage of the 'Const' identifier."),
            _ => return night_err!(Unimplemented, format!("Token '{tok:?}'")),
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

    fn parse_block(&mut self, guard: Option<Vec<String>>) -> Status {
        let mut block_queue = Vec::new();
        block_queue.push((self.instrs.len(), self.spans.len() - 1));

        while let Some((t, s)) = self.tokens.next() {
            self.spans.push(s);

            // Avoid excessive recursion. Technically I could just call `self.build_instr` without the if statement
            match t {
                Token::CloseCurly => {
                    // This will never be `None`
                    let (start, span_start) = block_queue.pop().unwrap();
                    let span_end = self.spans.len() - 1;
                    let mut block = self.instrs.split_off(start);
                    self.spans.push(Span::between(
                        &self.spans[span_start],
                        &self.spans[span_end],
                    ));

                    // Add guard to function if it is specified
                    if guard.is_some() && block_queue.is_empty() {
                        let guard = guard.clone().unwrap();
                        block.push_front(Instr::Guard(guard.clone(), span_start - 1));
                        block.push_back(Instr::GuardEnd(guard, span_start - 1));
                    }
                    push_instr!(Instr::PushFunc, BiFunction::from(block), self);

                    if block_queue.is_empty() {
                        break;
                    }
                }
                Token::OpenCurly => block_queue.push((self.instrs.len(), self.spans.len() - 1)),
                _ => self.build_instr(t)?,
            }
        }

        if block_queue.is_empty() {
            Ok(())
        } else {
            self.spans.push(Span::between(
                &self.spans[block_queue.pop().unwrap().1],
                self.spans.last().unwrap(),
            ));
            night_err!(Syntax, "Unbalanced block.")
        }
    }

    fn parse_define(&mut self) -> Status {
        let def_span = self.spans.len() - 1;
        let name;
        if let Some((Token::Symbol(s), span)) = self.tokens.next() {
            name = s.to_string();
            self.spans.push(span);
        } else {
            return night_err!(
                Syntax,
                "Expected a 'Symbol' to follow the 'Define' declaration."
            );
        }

        let name_span = self.spans.len() - 1;
        let (mut start, mut span) = self.tokens.next().ok_or(NightError::Syntax(
            "Definition cannot be empty.".to_string(),
        ))?;

        // Guard expression
        let mut guard = vec![];
        if start == Token::OpenParen {
            self.spans.push(span.clone());
            guard = self.parse_guard()?;
        }
        if !guard.is_empty() {
            (start, span) = self.tokens.next().ok_or(NightError::Syntax(
                "Definition cannot be empty.".to_string(),
            ))?;
        }

        // Const definition
        let is_const = start == Token::Pipe;
        let orig_len = self.instrs.len();

        if !is_const {
            self.spans.push(span);
            self.build_instr(start.clone())?;
        }

        // Body of definition is either a single block or a sequence of tokens followed by a newline/eof
        if start != Token::OpenCurly {
            self.parse_define_body(guard, def_span, orig_len, is_const)?;
        } else if !guard.is_empty() {
            if let Instr::PushFunc(mut f, s) = self.instrs.pop_back().unwrap() {
                f.instrs
                    .insert(0, Instr::Guard(guard.clone(), self.spans.len() - 2));
                f.instrs.push(Instr::GuardEnd(guard, self.spans.len() - 2));
                self.instrs
                    .push_back(Instr::PushFunc(BiFunction::from(f), s));
            } else {
                unreachable!();
            }
        }

        self.instrs
            .push_back(Instr::Push(Value::from(name), name_span));
        self.instrs
            .push_back(Instr::Internal(Builtin::Def, def_span));

        Ok(())
    }

    fn parse_guard(&mut self) -> Status<Vec<String>> {
        let mut guards = Vec::new();
        let span_start = self.spans.len() - 1;
        while let Some((tok, span)) = self.tokens.next() {
            self.spans.push(span);
            match tok {
                Token::CloseParen => break,
                Token::String(s) => {
                    if utils::is_one_word(s) {
                        guards.push(s.to_string());
                    } else {
                        return night_err!(Syntax, "Expected single word identifier.");
                    }
                }
                _ => return night_err!(Syntax, "Expected single word identifier."),
            }
        }

        if guards.is_empty() {
            night_err!(
                Syntax,
                "Guard expression should contain at least one word identifier."
            )
        } else {
            self.spans.push(Span::between(
                &self.spans[span_start],
                self.spans.last().unwrap(),
            ));
            Ok(guards)
        }
    }

    #[inline]
    fn parse_define_body(
        &mut self,
        guard: Vec<String>,
        start: usize,
        orig_len: usize,
        is_const: bool,
    ) -> Status {
        let mut def = VecDeque::new();
        while let Some((tok, s)) = self.tokens.next() {
            self.spans.push(s);
            if tok == Token::Newline || tok == Token::EOF {
                break;
            }
            self.build_instr(tok)?;
        }

        let len = self.instrs.len() - orig_len;
        for _ in 0..len {
            // Number of extra values tracked, so there will never be a panic
            def.push_front(self.instrs.pop_back().unwrap());
        }

        if is_const {
            let mut child = self.clone_child(def);
            child.exec();
            let mut scope = child.get_scope().borrow().to_owned();
            if scope.stack_len() != 1 {
                return Err(NightError::NothingToPop);
            }

            self.instrs
                .push_back(Instr::Push(scope.pop_value()?, start));
        } else {
            let span_start = def[0].get_span();
            let span_end = def[def.len() - 1].get_span();
            self.spans.push(Span::between(
                &self.spans[span_start],
                &self.spans[span_end],
            ));

            if !guard.is_empty() {
                def.push_front(Instr::Guard(guard.clone(), span_start - 1));
                def.push_back(Instr::GuardEnd(guard, span_start - 1));
            }
            push_instr!(Instr::PushFunc, BiFunction::from(def), self);
        }

        Ok(())
    }

    pub fn exec(&mut self) {
        while let Some(instr) = self.instrs.pop_front() {
            self.exec_instr(instr);
        }
    }

    #[inline]
    pub fn exec_instr(&mut self, instr: Instr) {
        use Instr::*;

        match instr {
            Push(v, _) => self.scope.borrow_mut().push_value(v),
            // When a symbol is defined as a function, it is executed in place
            PushSym(v, false, i) => {
                let definition = match self.scope.borrow().get_sym(v) {
                    Ok(inner) => inner.clone(),
                    Err(e) => error::error(e, self.spans[i].clone()),
                };

                match definition {
                    StackVal::Value(v) => self.scope.borrow_mut().push_value(v),
                    StackVal::Function(f) => {
                        for instr in f.instrs {
                            self.exec_instr(instr)
                        }
                    }
                }
            }
            PushSym(v, true, i) => {
                let mut s = self.scope.borrow_mut();
                let value = match s.get_reg(v) {
                    Ok(inner) => inner.clone(),
                    Err(e) => error::error(e, self.spans[i].clone()),
                };

                s.push(value)
            }
            PushFunc(f, _) => self.scope.borrow_mut().push(StackVal::Function(f)),
            Op(Operator::Call, i) => {
                if let Err(e) = self.exec_op_call() {
                    error::error(e, self.spans[i].clone());
                }
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
            Guard(guard, i) => {
                let mut s = self.scope.borrow_mut();
                for g in guard {
                    if let Err(e) = s.add_guard(g) {
                        error::error(e, self.spans[i].clone());
                    }
                }
            }
            GuardEnd(guard, _) => {
                let mut s = self.scope.borrow_mut();
                for g in guard {
                    s.rem_guard(g);
                }
            }
        }
    }

    fn exec_op_call(&mut self) -> Status {
        let top = self.scope.borrow_mut().pop()?;
        if let StackVal::Function(f) = top {
            for instr in f.instrs {
                self.exec_instr(instr);
            }
        } else {
            return night_err!(
                UnsupportedType,
                "Expected a function on the top of the stack."
            );
        }

        Ok(())
    }
}

impl Debug for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instr::Push(v, _) => write!(f, "{v}"),
            Instr::PushFunc(_, _) => write!(f, "<function>"),
            Instr::PushSym(s, false, _) => write!(f, "{s}"),
            Instr::PushSym(s, true, _) => write!(f, "${s}"),
            Instr::Op(o, _) => write!(f, "{o:?}"),
            Instr::Internal(b, _) => write!(f, "{b:?}"),
            Instr::Guard(syms, _) => write!(f, "<guard: {syms:?}>"),
            Instr::GuardEnd(syms, _) => write!(f, "<guard_end: {syms:?}>"),
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
