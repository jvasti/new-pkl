use logos::{Lexer, Span};

use crate::lexer::PklToken;

pub type ParseError = (String, Span);
type Result<T> = std::result::Result<T, ParseError>;

/* ANCHOR: statements */
/// Represent any valid Pkl value.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum PklStatement<'a> {
    Constant(&'a str, PklValue<'a>),
}
/* ANCHOR_END: statements */

/* ANCHOR: values */
/// Represent any valid Pkl value.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum PklValue<'a> {
    /// true or false.
    Bool(bool),
    /// Any floating point number.
    Float(f64),
    /// Any Integer.
    Int(i64),

    /// Any quoted string.
    String(&'a str),
    /// Any multiline string.
    MultiLineString(&'a str),
}
/* ANCHOR_END: values */

/* ANCHOR: statement */
/// Parse a token stream into a Pkl statement.
pub fn parse_pkl<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> Result<Vec<PklStatement<'a>>> {
    let mut statements = vec![];
    let mut is_newline = true;

    loop {
        if let Some(token) = lexer.next() {
            let result = match token {
                Ok(PklToken::Identifier(id)) => {
                    if !is_newline {
                        return Err((
                            "unexpected token here (context: global), expected newline".to_owned(),
                            lexer.span(),
                        ));
                    }

                    parse_const(lexer, id)
                }
                Ok(PklToken::IllegalIdentifier(id)) => {
                    if !is_newline {
                        return Err((
                            "unexpected token here (context: global), expected newline".to_owned(),
                            lexer.span(),
                        ));
                    }

                    parse_const(lexer, id)
                }
                Ok(PklToken::Space) => continue,
                Ok(PklToken::NewLine) => {
                    is_newline = true;
                    continue;
                }
                Err(e) => Err((e.to_string(), lexer.span())),
                _ => {
                    println!("error token: {:?}", token);
                    Err((
                        "unexpected token here (context: statement)".to_owned(),
                        lexer.span(),
                    ))
                }
            };

            is_newline = false;
            statements.push(result?);
        } else {
            break;
        }
    }

    Ok(statements)
}
/* ANCHOR_END: statement */

/* ANCHOR: value */
/// Parse a token stream into a Pkl value.
fn parse_value<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> Result<PklValue<'a>> {
    if let Some(token) = lexer.next() {
        match token {
            Ok(PklToken::Bool(b)) => Ok(PklValue::Bool(b)),
            Ok(PklToken::Int(i))
            | Ok(PklToken::OctalInt(i))
            | Ok(PklToken::HexInt(i))
            | Ok(PklToken::BinaryInt(i)) => Ok(PklValue::Int(i)),
            Ok(PklToken::Float(f)) => Ok(PklValue::Float(f)),
            Ok(PklToken::String(s)) => Ok(PklValue::String(s)),
            Ok(PklToken::MultiLineString(s)) => Ok(PklValue::MultiLineString(s)),
            Ok(PklToken::Space) => Ok(parse_value(lexer)?),
            Ok(PklToken::NewLine) => Ok(parse_value(lexer)?),
            Err(e) => Err((e.to_string(), lexer.span())),
            _ => Err((
                "unexpected token here (context: value)".to_owned(),
                lexer.span(),
            )),
        }
    } else {
        Err(("empty values are not allowed".to_owned(), lexer.span()))
    }
}
/* ANCHOR_END: value */

/* ANCHOR: const */
/// Parse a token stream into a Pkl const Statement.
fn parse_const<'a>(lexer: &mut Lexer<'a, PklToken<'a>>, name: &'a str) -> Result<PklStatement<'a>> {
    if let Some(token) = lexer.next() {
        match token {
            Ok(PklToken::EqualSign) => {
                let value = parse_value(lexer)?;

                Ok(PklStatement::Constant(name, value))
            }
            Ok(PklToken::Space) => Ok(parse_const(lexer, name)?),
            Ok(PklToken::NewLine) => Ok(parse_const(lexer, name)?),
            Err(e) => Err((e.to_string(), lexer.span())),
            _ => Err((
                "unexpected token here (context: constant)".to_owned(),
                lexer.span(),
            )),
        }
    } else {
        Err(("Expected '='".to_owned(), lexer.span()))
    }
}
/* ANCHOR_END: const */
