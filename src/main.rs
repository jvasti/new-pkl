use lexer::PklToken;
use logos::Logos;

mod lexer;

fn main() {
    let lex = PklToken::lexer("`Hello ` world = 2_222.334e2");

    for i in lex {
        println!("{:?}", i);
    }
}
