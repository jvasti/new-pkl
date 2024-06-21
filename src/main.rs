use std::time::Instant;

use lexer::PklToken;
use logos::Logos;
use parser::{parse_pkl, ParseError};

mod lexer;
mod parser;

fn main() -> Result<(), ParseError> {
    let src = "`Hello` = \"hello\"
test = 222_333.3e-4
b = true
octal = 0o1_237
hex = 0x129_EF2444443
binary = 0b1_010_10100011111101010101

multiline = \"\"\"
Although the Dodo is extinct,
the species will be remembered.
efefefefef
\"\"\"
";

    let time = Instant::now();
    let s = src.repeat(1000);
    let mut lexer = PklToken::lexer(&s);

    let filename = String::from("main.pkl");

    match parse_pkl(&mut lexer) {
        Ok(value) => println!("{:#?}", value),
        Err((msg, span)) => {
            use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

            let mut colors = ColorGenerator::new();

            let a = colors.next();

            Report::build(ReportKind::Error, &filename, 12)
                .with_message("Invalid Pkl".to_string())
                .with_label(
                    Label::new((&filename, span))
                        .with_message(msg)
                        .with_color(a),
                )
                .finish()
                .eprint((&filename, Source::from(src)))
                .unwrap();
        }
    }

    println!(
        "{}ms to parse {} chars",
        time.elapsed().as_millis(),
        s.len()
    );

    Ok(())
}
