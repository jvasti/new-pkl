use std::time::Instant;

use lexer::PklToken;
use logos::Logos;
use parser::{parse_pkl, ParseError, PklStatement};

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

identifier_var = multiline

bird {
  name = \"Common wood pigeon\"
  diet = \"Seeds\"
  taxonomy {
    species = \"Columba palumbus\"
  }
}

parrot = (pigeon) {
  name = \"Parrot\"
}

dodo {
  name = \"Dodo\"
} {
  extinct = true
} {
  test = false
}
";

    let src = src.repeat(10000);
    let time = Instant::now();
    let mut lexer = PklToken::lexer(&src);

    match parse_pkl(&mut lexer) {
        Ok(value) => {
            // println!("{:#?}", value);
            let PklStatement::Constant(_, val, _) = &value[10];
            println!("{:?}", val);
            println!("{}", &lexer.source()[val.span()])
        }
        Err((msg, span)) => {
            println!("Error: {} at {:?}", msg, span)
            // use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

            // let mut colors = ColorGenerator::new();

            // let a = colors.next();
            // let filename = String::from("main.pkl");

            // Report::build(ReportKind::Error, &filename, 12)
            //     .with_message("Invalid Pkl".to_string())
            //     .with_label(
            //         Label::new((&filename, span))
            //             .with_message(msg)
            //             .with_color(a),
            //     )
            //     .finish()
            //     .eprint((&filename, Source::from(src)))
            //     .unwrap();
        }
    }

    println!(
        "{}ms to parse {} chars",
        time.elapsed().as_millis(),
        src.len()
    );

    Ok(())
}
