use night::interpreter::Night;

fn main() {
    // Simulate execution of a program for testing
    const TEST: &'static str = r#"
        -> value | 4 5 +
        -> double . +
        -> do_stuff {
            1 2 3 4 5 + * . +
            : inc
        }
        4 7 * 3 6 9 add -- this is a comment
        print
        ; 4 . / inc :
        value double do_stuff
    "#;

    let mut lex = night::lexer::Lexer::new(TEST);
    let tokens = lex.tokenize();
    let mut night = Night::new(TEST, tokens);

    night.init();
    println!("{night}");

    night.exec();
    print!("Stack:\n{}", night.get_scope().borrow());
}
