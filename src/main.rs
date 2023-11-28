use night::interpreter::Night;

fn main() {
    // Simulate execution of a program for testing

    const TEST: &'static str = r#"
        4 7 * 3 6 9 add -- this is a comment
        print
        ; 4 . / :
    "#;
    let mut lex = night::lexer::Lexer::new(TEST);
    let tokens = lex.tokenize();
    let mut night = Night::new(TEST, tokens);

    night.init();
    night.exec(); // 15 should be printed
    print!("Stack:\n{}", night.get_scope().borrow()); // Stack should be [1, 28] from bottom to top
}
