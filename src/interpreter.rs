use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{self, Debug, Display};
use std::rc::Rc;
use std::vec::IntoIter;

use crate::builtin::{Builtin, Intrinsic as Intr, Operator, BUILTIN_MAP};
use crate::lexer::{LexTok, Token};
use crate::scope::{Scope, ScopeInternal, StackVal};
use crate::utils::error::{self, night_err, NightError, Span, Status};
use crate::utils::function::{BlockFunc, Generable, SingleFunc};
use crate::value::Value;

#[derive(Clone)]
pub enum Instr {
    Push(Value, usize),
    PushFunc(Rc<dyn Generable>, usize),
    PushSym(String, bool, usize),
    Op(Operator, usize),
    Internal(Builtin, usize),
    Intrinsic(Intr, usize),
    Guard(Vec<String>, usize),
    GuardEnd(Vec<String>, usize),
    Block(Vec<String>, usize),
    Unblock(Vec<String>, usize),
    EndCallback,
}

impl Instr {
    pub fn get_span(&self) -> usize {
        match self {
            Instr::Push(_, s) => *s,
            Instr::PushFunc(_, s) => *s,
            Instr::PushSym(_, _, s) => *s,
            Instr::Op(_, s) => *s,
            Instr::Internal(_, s) => *s,
            Instr::Intrinsic(_, s) => *s,
            Instr::Guard(_, s) => *s,
            Instr::GuardEnd(_, s) => *s,
            Instr::Block(_, s) => *s,
            Instr::Unblock(_, s) => *s,
            Instr::EndCallback => usize::MAX,
        }
    }
}

pub struct Night {
    input: Box<str>,
    tokens: IntoIter<LexTok>,
    spans: Vec<Span>,

    // It's easier to use a deque, since I can use a while `pop_back` and then easily modify in between iterations
    instrs: VecDeque<Instr>,
    scope: Scope,
    callback: Vec<usize>,
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

impl Night {
    pub fn new() -> Self {
        Self {
            input: "".into(),
            tokens: vec![].into_iter(),
            spans: vec![],
            instrs: VecDeque::new(),
            scope: Rc::new(RefCell::new(ScopeInternal::create())),
            callback: vec![],
        }
    }

    pub fn clone_child(&self, instrs: impl Into<VecDeque<Instr>>) -> Self {
        Self {
            input: self.input.clone(),
            tokens: vec![].into_iter(),
            spans: self.spans.clone(),
            instrs: instrs.into(),
            scope: Rc::new(RefCell::new(self.scope.borrow().to_owned().clone())),
            callback: vec![],
        }
    }

    pub fn push_new_code(&mut self, code: &str, tokens: Vec<LexTok>) {
        self.input = code.into();
        self.tokens = tokens.into_iter();
        self.init();
    }

    pub fn get_scope(&self) -> Scope {
        self.scope.clone()
    }

    pub fn inject_code(&mut self, tokens: Vec<LexTok>) {
        let mut tokens = tokens.into_iter();
        std::mem::swap(&mut self.tokens, &mut tokens);
        self.init();
        self.exec();
        self.tokens = tokens;
    }

    #[inline]
    fn span_between(&mut self, left: usize, right: usize) {
        let span = Span::between(&self.spans[left], &self.spans[right]);
        self.spans.push(span);
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
            Token::Number(n) => {
                if n.contains('.') {
                    push_instr!(Instr::Push, Value::from(n.parse::<f64>()?), self)
                } else {
                    push_instr!(Instr::Push, Value::from(n.parse::<i64>()?), self)
                }
            }
            Token::String(s) => push_instr!(Instr::Push, Value::from(s.to_string()), self),
            Token::Register(s) => push_instr!(Instr::PushSym, s.to_string(), true, self),
            Token::Op(Operator::Call) => push_instr!(Instr::Intrinsic, Intr::Call, self),
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
            Token::DefineSym => {
                if self.spans.len() > 1 {
                    let mut i = self.spans.len() - 2;
                    loop {
                        match self.spans[i].as_lit() {
                            b"\n" => break,
                            s if s.len() > 1 || !s[0].is_ascii_whitespace() => {
                                return night_err!(
                                    Syntax,
                                    "Definition must begin at the start of a line."
                                )
                            }
                            _ => {
                                i -= 1;
                                continue;
                            }
                        }
                    }
                }
                self.parse_define()?
            }
            Token::Exclamation => {
                if let Some(Instr::PushSym(reg, true, span)) = self.instrs.pop_back() {
                    push_instr!(Instr::Intrinsic, Intr::DefineRegister, self);
                    self.instrs.push_back(Instr::PushSym(reg, true, span))
                } else {
                    return night_err!(Syntax, "The '!' token must follow a valid register.");
                }
            }
            Token::Symbol(_) => self.instrs.push_back(self.maybe_builtin(tok)),
            Token::Newline | Token::EOF => {} // skip
            Token::Pipe => {
                let ident = self.instrs.pop_back().ok_or(NightError::Syntax(
                    "Register block statement requires a preceeding literal.".to_string(),
                ))?;
                match ident {
                    Instr::Push(value, _) if value.is_str() => {
                        let name = value.as_str()?;
                        push_instr!(Instr::Block, vec![name], self)
                    }
                    _ => return night_err!(Syntax, "Register block statement requires a valid preceeding literal [word/string/array of strings]."),
                }
            }
            Token::AtSign => {
                let instr = self.instrs.pop_back().ok_or(NightError::Syntax(
                    "Singleton block statement missing preceeding instruction.".to_string(),
                ))?;
                self.span_between(self.spans.len() - 2, self.spans.len() - 1);
                push_instr!(Instr::PushFunc, Rc::new(SingleFunc::from(instr)), self)
            }
            _ => return night_err!(Unimplemented, format!("Token '{tok:?}'")),
        }

        Ok(())
    }

    fn maybe_builtin(&self, tok: Token) -> Instr {
        if let Token::Symbol(s) = tok {
            let s = s.as_ref();
            if let Some(i) = Intr::from_name(s) {
                Instr::Intrinsic(i, self.spans.len() - 1)
            } else if let Some(&b) = BUILTIN_MAP.get(s) {
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
                    self.span_between(span_start, span_end);

                    // Add guard to function if it is specified
                    if guard.is_some() && block_queue.is_empty() {
                        let guard = guard.clone().unwrap();
                        block.push_front(Instr::Guard(guard.clone(), span_start - 1));
                        block.push_back(Instr::GuardEnd(guard, span_start - 1));
                    }
                    push_instr!(Instr::PushFunc, Rc::new(BlockFunc::from(block)), self);

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
            self.span_between(block_queue.pop().unwrap().1, self.spans.len() - 1);
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

        if start == Token::Newline {
            self.span_between(self.spans.len() - 2, self.spans.len() - 1);
            return night_err!(Syntax, "Definition cannot start with a newline.");
        }

        // Const definition
        let is_const = start == Token::Pipe;
        let orig_len = self.instrs.len();
        if !is_const {
            self.spans.push(span);
            self.build_instr(start.clone())?;
        }

        if !guard.is_empty() && is_const {
            self.span_between(self.spans.len() - 2, self.spans.len() - 1);
            return night_err!(Syntax, "Guarded definitions cannot be const.");
        }
        self.parse_define_body(guard, def_span, orig_len, is_const)?;

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
                Token::Symbol(s) => {
                    let s = s.as_ref();
                    guards.push(s.to_string());
                }
                _ => return night_err!(Syntax, "Expected literal identifier."),
            }
        }

        if guards.is_empty() {
            night_err!(
                Syntax,
                "Guard expression should contain at least one word identifier."
            )
        } else {
            self.span_between(span_start, self.spans.len() - 1);
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
        let mut final_span = Span::empty();
        while let Some((tok, s)) = self.tokens.next() {
            if tok == Token::Newline || tok == Token::EOF {
                final_span = s;
                break;
            }
            self.spans.push(s);
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
        // TODO: Finish to fix single block defs
        } else if def.len() == 1 {
            if let Instr::PushFunc(f, s) = &def[0] {
                let mut instrs = Vec::with_capacity(f.len() + 2);
                instrs.push(Instr::Guard(guard.clone(), self.spans.len() - 2));
                instrs.extend(f.gen_instrs(*s));
                instrs.push(Instr::GuardEnd(guard, self.spans.len() - 2));
                self.instrs
                    .push_back(Instr::PushFunc(Rc::new(BlockFunc::from(instrs)), *s));
            } else {
                let span = def[0].get_span();
                if !guard.is_empty() {
                    def.push_front(Instr::Guard(guard.clone(), span - 1));
                    def.push_back(Instr::GuardEnd(guard, span - 1));
                    push_instr!(Instr::PushFunc, Rc::new(BlockFunc::from(def)), self);
                } else {
                    self.instrs.push_back(def.pop_back().unwrap());
                }
            }
        } else {
            let span_start = def[0].get_span();
            let span_end = def[def.len() - 1].get_span();
            self.span_between(span_start, span_end);

            if !guard.is_empty() {
                def.push_front(Instr::Guard(guard.clone(), span_start - 1));
                def.push_back(Instr::GuardEnd(guard, span_start - 1));
            }
            push_instr!(Instr::PushFunc, Rc::new(BlockFunc::from(def)), self);
        }

        self.spans.push(final_span);
        Ok(())
    }

    #[inline]
    pub fn exec(&mut self) {
        while let Some(instr) = self.instrs.pop_front() {
            let span = instr.get_span();
            if let Err(e) = self.exec_instr(instr) {
                // If there is an existing trace, print that too
                if !self.callback.is_empty() {
                    let trace = self
                        .callback
                        .iter()
                        .rev()
                        .map(|&i| self.spans[i].clone())
                        .collect();
                    error::error_with_trace(e, self.spans[span].clone(), trace);
                } else {
                    error::error(e, self.spans[span].clone());
                }
            }
        }
    }

    // Unroll the loop to avoid excessive recursion
    #[inline]
    pub fn exec_fn(&mut self, def: Vec<Instr>, from: usize) {
        self.callback.push(from);
        self.instrs.push_front(Instr::EndCallback);
        for instr in def.into_iter().rev() {
            self.instrs.push_front(instr);
        }
    }

    #[inline]
    pub fn exec_instr(&mut self, instr: Instr) -> Status {
        use Instr::*;

        match instr {
            Push(v, _) => self.scope.borrow_mut().push_value(v),
            // When a symbol is defined as a function, it is executed in place
            PushSym(v, false, i) => {
                let definition = self.scope.borrow().get_sym(v).cloned()?;
                match definition {
                    StackVal::Value(v) => self.scope.borrow_mut().push_value(v),
                    StackVal::Function(f) => self.exec_fn(f.gen_instrs(i), i),
                }
            }
            PushSym(v, true, _) => {
                let mut s = self.scope.borrow_mut();
                let value = s.get_reg(v).cloned()?;
                s.push(value)
            }
            PushFunc(f, _) => self.scope.borrow_mut().push(StackVal::Function(f)),
            Intrinsic(intr, i) => self.exec_intrinsic(intr, i)?,
            Op(o, _) => o.call(self.scope.clone())?,
            Internal(b, _) => b.call(self.scope.clone())?,
            Guard(guard, _) => {
                let mut s = self.scope.borrow_mut();
                for g in guard {
                    s.add_guard(g)?;
                }
            }
            GuardEnd(guard, _) => {
                let mut s = self.scope.borrow_mut();
                for g in guard {
                    s.rem_guard(g)?;
                }
            }
            Block(guard, i) => {
                if self.instrs.is_empty() {
                    return night_err!(Runtime, "Block expression must preceed some operation.");
                }
                self.instrs.insert(1, Instr::Unblock(guard.clone(), i));
                let mut s = self.scope.borrow_mut();
                for g in guard {
                    s.add_block(g)?
                }
            }
            Unblock(guard, _) => {
                let mut s = self.scope.borrow_mut();
                for g in guard {
                    s.rem_block(g);
                }
            }
            EndCallback => {
                self.callback.pop();
            }
        }

        Ok(())
    }

    fn exec_intrinsic(&mut self, intr: Intr, from: usize) -> Status {
        let scope = self.scope.clone();
        match intr {
            Intr::Call => self.exec_intr_call(from),
            Intr::If => self.exec_intr_if(from),
            Intr::Loop => self.exec_intr_loop(from),
            Intr::DefineRegister => self.exec_intr_defr(from),
            Intr::StackDump => {
                println!("--- STACK DMP: ---\n{}\n---            ---", scope.borrow());
                Ok(())
            }
            Intr::SymDump => {
                scope.borrow().dump_symbols();
                Ok(())
            }
        }
    }

    fn exec_intr_call(&mut self, from: usize) -> Status {
        let scope = self.scope.clone();
        let def = scope.borrow_mut().pop()?.as_fn()?;
        self.exec_fn(def.gen_instrs(from), from);
        Ok(())
    }

    // This can be implemented as a fn instead of an intrinsic at this point.
    // See Factor's implementation of `times`, the `night` version can be implemented
    // with the same logic.
    fn exec_intr_loop(&mut self, from: usize) -> Status {
        let mut s = self.scope.borrow_mut();
        let def = s.pop()?.as_fn()?;
        let count = s.pop_value()?.as_int()?;
        drop(s);
        if count < 0 {
            return night_err!(Runtime, "'loop' can only take a positive integer.");
        }

        for _ in 0..count {
            self.exec_fn(def.gen_instrs(from), from);
        }
        Ok(())
    }

    fn exec_intr_if(&mut self, from: usize) -> Status {
        let mut s = self.scope.borrow_mut();
        let false_def = s.pop()?.as_fn()?;
        let true_def = s.pop()?.as_fn()?;
        let cond = s.pop_value()?.as_bool()?;
        drop(s);
        if cond {
            self.exec_fn(true_def.gen_instrs(from), from);
        } else {
            self.exec_fn(false_def.gen_instrs(from), from);
        }
        Ok(())
    }

    fn exec_intr_defr(&mut self, defr_span: usize) -> Status {
        let scope = self.scope.clone();
        let mut s = scope.borrow_mut();
        if let Some(Instr::PushSym(reg, true, reg_span)) = self.instrs.pop_front() {
            self.span_between(reg_span, defr_span);
            let between = self.spans.len() - 1;
            self.spans.swap(defr_span, between);
            let top = s.pop()?;
            s.def_reg(reg, top)?;
            Ok(())
        } else {
            unreachable!()
        }
    }
}

impl Debug for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instr::Push(v, _) => write!(f, "Push({v})"),
            Instr::PushFunc(_, _) => write!(f, "Push(<function>)"),
            Instr::PushSym(s, false, _) => write!(f, "Exec({s})"),
            Instr::PushSym(s, true, _) => write!(f, "Push(${s})"),
            Instr::Op(o, _) => write!(f, "{o:?}"),
            Instr::Internal(b, _) => write!(f, "{b:?}"),
            Instr::Intrinsic(i, _) => write!(f, "Intrinsic({i:?}"),
            Instr::Guard(syms, _) => write!(f, "<guard: {syms:?}>"),
            Instr::GuardEnd(syms, _) => write!(f, "<guard_end: {syms:?}>"),
            Instr::Block(syms, _) => write!(f, "<block: {syms:?}>"),
            Instr::Unblock(syms, _) => write!(f, "<unblock: {syms:?}>"),
            Instr::EndCallback => unreachable!(),
        }
    }
}

impl Display for Night {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Night")
            .field("instrs", &self.instrs)
            .field("stack", &format!("{}", self.scope.borrow()))
            .finish()
    }
}
