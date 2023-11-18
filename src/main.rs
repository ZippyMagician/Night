use night::utils::function::InlineFunction;

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

    let s = night::scope::Scope;
    let mut op_add = night::operator::Operator::Add;

    let status = op_add.call(s);
    println!("{:?}", status);
}
