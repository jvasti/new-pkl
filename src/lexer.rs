use logos::Logos;

#[derive(Debug, PartialEq, PartialOrd, Logos)]
#[logos(skip r"[\t]+")]
pub enum PklToken<'a> {
    #[token("_", priority = 3)]
    BlankIdentifier,
    #[token(" ")]
    Space,
    #[token("\n")]
    NewLine,
    #[token("=")]
    EqualSign,
    #[token("true")]
    True,
    #[token("false")]
    False,

    #[regex(r"-?\d(?:_?\d)*", |lex| {
        let raw = lex.slice();
        // Remove underscores for parsing
        let clean_raw: String = raw.chars().filter(|&c| c != '_').collect();
        clean_raw.parse::<i64>().unwrap()
    }, priority = 3)]
    Int(i64),

    #[regex(r"-?0x[0-9a-fA-F]+(?:_?[0-9a-fA-F])*", |lex| {
        let raw = lex.slice();
        // Check for the optional minus sign
        let (is_negative, hex_str) = if raw.starts_with('-') {
            (true, &raw[3..]) // Skip "-0x"
        } else {
            (false, &raw[2..]) // Skip "0x"
        };

        // Remove underscores for parsing
        let clean_hex: String = hex_str.chars().filter(|&c| c != '_').collect();
        let value = i64::from_str_radix(&clean_hex, 16).unwrap();

        if is_negative {
            -value
        } else {
            value
        }
    })]
    HexInt(i64),

    #[regex(r"-?0b[01]+(?:_?[01])*", |lex| {
        let raw = lex.slice();
        // Check for the optional minus sign
        let (is_negative, hex_str) = if raw.starts_with('-') {
            (true, &raw[3..]) // Skip "-0b"
        } else {
            (false, &raw[2..]) // Skip "0b"
        };

        // Remove underscores for parsing
        let clean_hex: String = hex_str.chars().filter(|&c| c != '_').collect();
        let value = i64::from_str_radix(&clean_hex, 2).unwrap();

        if is_negative {
            -value
        } else {
            value
        }
    })]
    BinaryInt(i64),

    #[regex(r"-?0o[0-7]+(?:_?[0-7])*", |lex| {
        let raw = lex.slice();
        // Check for the optional minus sign
        let (is_negative, hex_str) = if raw.starts_with('-') {
            (true, &raw[3..]) // Skip "-0o"
        } else {
            (false, &raw[2..]) // Skip "0o"
        };

        // Remove underscores for parsing
        let clean_hex: String = hex_str.chars().filter(|&c| c != '_').collect();
        let value = i64::from_str_radix(&clean_hex, 8).unwrap();

        if is_negative {
            -value
        } else {
            value
        }
    })]
    OctalInt(i64),

    #[regex(r"NaN|-?Infinity|(-?(?:0|[1-9]+(?:_?\d)*)?(?:\.\d+(?:_?\d)*)?(?:[eE][+-]?\d+(?:_?\d)*)?)", |lex| {
        let raw = lex.slice();

        if raw == "NaN" {
            return std::f64::NAN;
        }
        if raw == "Infinity" {
            return std::f64::INFINITY;
        }
        if raw == "-Infinity" {
            return std::f64::NEG_INFINITY;
        }

        let clean_raw: String = raw.chars().filter(|&c| c != '_').collect();
        clean_raw.parse::<f64>().unwrap()
    }, priority = 2)]
    Float(f64),

    #[regex(r#"(\$|_\d*)?[a-zA-Z]\w+"#, |lex| lex.slice())]
    Identifier(&'a str),
    #[regex(r#"`([^`\\]|\\[`\\bnfrt]|\\u\{[a-fA-F0-9]+})*`"#, |lex| {let raw=lex.slice();&raw[1..raw.len()-1]})]
    IllegalIdentifier(&'a str),

    #[regex(r#""([^"\\]|\\["\\bnfrt]|\\u\{[a-fA-F0-9]+})*""#, |lex| let raw=lex.slice();&raw[1..raw.len()-1])]
    String(&'a str),
    #[regex(r#""""\n([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*\n""""#, |lex| let raw=lex.slice();&raw[3..raw.len()-3])]
    MultiLineString(&'a str),
}
