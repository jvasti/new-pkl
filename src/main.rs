use lexer::PklToken;
use logos::Logos;

mod lexer;

fn main() {
    let lex = PklToken::lexer(
        "`Hello` = \"hello\"
test = 222_333.3e-4
b = true
octal = 0o1_237
hex = 0x129_EF2
binary = 0o1_010_101
",
    );

    for i in lex {
        println!("{:?}", i);
    }
}
