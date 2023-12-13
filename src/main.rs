use night::interpreter::Night;
use night::lexer::{Lexer, Token};
use night::utils;

fn main() {
    // Simulate execution of a program for testing
    /*const TEST: &'static str = r#"
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
    "#;*/

    /*const TEST: &'static str = r#"
    -> dip (:dip) : :dip ! ; ? $dip
    3 6 {1 +} dip
    :dip undef ;
    -> mults (:a) :a ! 9 {. $a +} loop
    7 mults
    "#;*/

    // TODO: Test once more logical ops are implemented
    const TEST: &'static str = r#"
    -> fib . . 1 = : 0 = or ~ { . dec fib : 2 - fib + } if
    10 fib
    1 3.5 /
    3 6 {inc} dip
    :test "this is a string" 'a :b:c
    "#;

    let mut lex = Lexer::new(TEST);
    let tokens = lex.tokenize();
    let mut night = Night::new(TEST, tokens.clone());

    utils::define_fn(
        &mut night,
        r#"
        -> dip (:top) : :top ! ; :top | ? $top
        "#,
    );

    println!(
        "{:?}\n---",
        tokens.into_iter().map(|(n, _)| n).collect::<Vec<Token>>()
    );

    night.init();
    println!("{night}\n---");

    night.exec();
    print!("Stack:\n{}", night.get_scope().borrow());
}
