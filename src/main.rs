use night::interpreter::Night;

fn main() {
    // Simulate execution of a program for testing
    const TEST: &'static str = r#"
        4 7 * 3 6 9 add -- this is a comment
        -> value | 4 5 +
        print
        ; 4 . / inc :
        value . *
    "#;

    let mut lex = night::lexer::Lexer::new(TEST);
    let tokens = lex.tokenize();
    let mut night = Night::new(TEST, tokens);

    night.init();
    // println!("{night}"); // print state for debugging
    night.exec(); // 15 should be printed
    print!("Stack:\n{}", night.get_scope().borrow()); // Stack should be [2, 28, 81] from bottom to top
}
