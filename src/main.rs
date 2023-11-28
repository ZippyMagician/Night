use night::interpreter::Night;

fn main() {
    // Simulate execution of "TEST"

    const TEST: &'static str = "4 7 * 3 6 9 add -- this is a comment\nprint 0";
    let mut lex = night::lexer::Lexer::new(TEST);
    let tokens = lex.tokenize();
    let mut night = Night::init(TEST, tokens);

    night.build_instrs();
    night.exec_all(); // 15 should be printed
    print!("Stack:\n{}", night.get_scope().borrow()); // Stack should be [11, 3, 0] from bottom to top
}
