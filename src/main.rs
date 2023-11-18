use night::interpreter::{Instr, Night};
use night::lexer::{Token, Span};
use night::value::Value;

fn main() {
    let program = r#"
    5 :x ! . * : $x : -
    y <- 4 9 +
    "hello" 'c '@
    "#;
    let mut lex = night::lexer::Lexer::new(program);
    let tokens = lex
        .tokenize()
        .into_iter()
        .map(|(t, _)| t)
        .collect::<Vec<_>>();
    println!("{:?}", tokens);

    let program = "4 7 +";
    let mut night = Night::init(program, vec![]);

    // Simulate execution
    night.exec(Instr::Push(Value::from(4), 0));
    night.exec(Instr::Push(Value::from(7), 2));
    night.exec(Instr::Op(night::builtin::Operator::Add, 4));
    let print = night.maybe_builtin((Token::Symbol("print"), Span::empty()));
    night.exec(print);
}
