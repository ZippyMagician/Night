use crate::utils::token::Token;

pub mod lexer;
pub mod utils;

fn main() {
    let mut lex = lexer::Lexer::new("5:x!.*$x:- 5");
    let tokens = lex
        .tokenize()
        .into_iter()
        .map(|(t, _)| t)
        .collect::<Vec<Token>>();
    println!("{:?}", tokens);
}
