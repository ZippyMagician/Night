pub mod lexer;
pub mod operator;
pub mod utils;

use lexer::{Lexer, Token};

fn main() {
    let mut lex = Lexer::new("5:x!.*$x:- 5");
    let tokens = lex
        .tokenize()
        .into_iter()
        .map(|(t, _)| t)
        .collect::<Vec<Token>>();
    println!("{:?}", tokens);
}
