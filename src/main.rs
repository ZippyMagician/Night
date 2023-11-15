pub mod lexer;
pub mod operator;
pub mod utils;

use lexer::{Lexer, Token};

fn main() {
    let program = r#"
    5 :x ! . * : $x : -
    y <- 4 9 +
    "hello" 'c '@
    "#;
    let mut lex = Lexer::new(program);
    let tokens = lex
        .tokenize()
        .into_iter()
        .map(|(t, _)| t)
        .collect::<Vec<Token>>();
    println!("{:?}", tokens);
}
