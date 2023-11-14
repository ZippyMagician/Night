pub mod lexer;
pub mod operator;
pub mod utils;

use lexer::{Lexer, Token};

fn main() {
    let mut lex = Lexer::new(r#"5 :x ! . * "hello world" : $x : -"#);
    let tokens = lex
        .tokenize()
        .into_iter()
        .map(|(t, _)| t)
        .collect::<Vec<Token>>();
    println!("{:?}", tokens);
}
