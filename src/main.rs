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

    /*const TEST: &'static str = r#"
    -> fib . 2 >= { { dec fib } { 2 - fib } bi + } when
    10 fib -- 55
    1 3.5 / -- 0.2857143
    1 2 3 4 { inc { 4 + } dip } dip -- 1 6 4 4
    3 4 +@ curry ? -- 7
    0 1 and@ not@ bind ? -- 1
    "#;*/

    const TEST: &'static str = r#"
    -> fib { 0 1 } dip { +@ ;@ bi2 } loop ;
    3 4 +@ *@ bi2
    5 3 { 0 : - } dec@ fork
    10 fib 55 = "pass" "fail" choose

    -> for_range {
        over2 <
        { (:I) { { ; :I ! ; } dip ? } keep3 inc@ dip2 for_range }
        pop3@ if
    }
    1 11 { $I 2 * } for_range
    "#;

    let mut lex = Lexer::new(TEST);
    let tokens = lex.tokenize();
    let mut night = Night::new(TEST, tokens.clone());

    utils::define_fns(
        &mut night,
        r#"
        -> rotn 1 - {} { { dip : } curry } swpd loop ?
        -> over2 pick pick
        -> dip (:top) : :top ! ; :top | ? $top
        -> dip2 : dip@ dip
        -> dip3 : dip2@ dip
        -> keep over ?@ dip
        -> keep2 dup2@ dip dip2
        -> keep3 dup3@ dip dip3
        -> bi keep@ dip ?
        -> bi2 keep2@ dip ?
        -> fork dip@ dip ?
        -> fork2 dip2@ dip ?
        -> when : ?@ ;@ if
        -> unless : ;@ ?@ if
        -> choose 3 rotn ;@ nip@ if
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
