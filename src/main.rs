use night::interpreter::{Instr, Night};
use night::lexer::{Span, Token};
use night::value::Value;

fn main() {
    // Simulate execution of "TEST"

    const TEST: &'static str = "4 7 + print 3 6 9 add print print -- this is a comment\n0";
    let mut lex = night::lexer::Lexer::new(TEST);
    let tokens = lex
        .tokenize()
        .into_iter()
        .map(|(t, _)| t)
        .collect::<Vec<_>>();
    println!("{:?}", tokens);
    let mut night = Night::init(TEST, vec![]);

    let add = night.maybe_builtin((Token::Symbol("add"), Span::empty()));
    let print = night.maybe_builtin((Token::Symbol("print"), Span::empty()));

    night.exec(Instr::Push(Value::from(4), 0));
    night.exec(Instr::Push(Value::from(7), 2));
    night.exec(Instr::Op(night::builtin::Operator::Add, 4));
    night.exec(print.clone());
    night.exec(Instr::Push(Value::from(3), 12));
    night.exec(Instr::Push(Value::from(6), 14));
    night.exec(Instr::Push(Value::from(9), 16));
    night.exec(add.clone());
    night.exec(print.clone());
    night.exec(print.clone());
    night.exec(Instr::Push(Value::from(0), 59));
}
