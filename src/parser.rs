use std::{collections::HashMap, ops::Range};

use logos::{Lexer, Span};

use crate::lexer::PklToken;

pub type ParseError = (String, Span);
type Result<T> = std::result::Result<T, ParseError>;

/* ANCHOR: statements */
/// Represent any valid Pkl value.
#[derive(Debug, PartialEq, Clone)]
pub enum PklStatement<'a> {
    Constant(&'a str, PklValue<'a>, Range<usize>),
}
/* ANCHOR_END: statements */

/* ANCHOR: values */
/// Represent any valid Pkl value.
#[derive(Debug, PartialEq, Clone)]
pub enum PklValue<'a> {
    /// true or false.
    Bool(bool, Range<usize>),
    /// Any floating point number.
    Float(f64, Range<usize>),
    /// Any Integer.
    Int(i64, Range<usize>),

    /// Any quoted string.
    String(&'a str, Range<usize>),
    /// Any multiline string.
    MultiLineString(&'a str, Range<usize>),

    /// An object.
    Object(HashMap<&'a str, PklValue<'a>>, Range<usize>),
}
/* ANCHOR_END: values */

impl<'a> PklStatement<'a> {
    pub fn span(&self) -> Range<usize> {
        match self {
            PklStatement::Constant(_, _, rng) => rng.clone(),
        }
    }
}

impl<'a> PklValue<'a> {
    pub fn span(&self) -> Range<usize> {
        match self {
            PklValue::Int(_, rng)
            | PklValue::Bool(_, rng)
            | PklValue::Float(_, rng)
            | PklValue::Object(_, rng)
            | PklValue::String(_, rng)
            | PklValue::MultiLineString(_, rng) => rng.clone(),
        }
    }
}

/* ANCHOR: statement */
/// Parse a token stream into a Pkl statement.
pub fn parse_pkl<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> Result<Vec<PklStatement<'a>>> {
    let mut statements = vec![];
    let mut is_newline = true;

    loop {
        if let Some(token) = lexer.next() {
            let result = match token {
                Ok(PklToken::Identifier(id)) | Ok(PklToken::IllegalIdentifier(id)) => {
                    if !is_newline {
                        return Err((
                            "unexpected token here (context: global), expected newline".to_owned(),
                            lexer.span(),
                        ));
                    }

                    parse_const(lexer, id)
                }
                Ok(PklToken::Space) => continue,
                Ok(PklToken::DocComment(_))
                | Ok(PklToken::LineComment(_))
                | Ok(PklToken::MultilineComment(_)) => continue,
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
            Ok(PklToken::Bool(b)) => Ok(PklValue::Bool(b, lexer.span())),
            Ok(PklToken::Int(i))
            | Ok(PklToken::OctalInt(i))
            | Ok(PklToken::HexInt(i))
            | Ok(PklToken::BinaryInt(i)) => Ok(PklValue::Int(i, lexer.span())),
            Ok(PklToken::Float(f)) => Ok(PklValue::Float(f, lexer.span())),
            Ok(PklToken::String(s)) => Ok(PklValue::String(s, lexer.span())),
            Ok(PklToken::MultiLineString(s)) => Ok(PklValue::MultiLineString(s, lexer.span())),

            Ok(PklToken::Space) => Ok(parse_value(lexer)?),
            Ok(PklToken::NewLine) => Ok(parse_value(lexer)?),
            Ok(PklToken::DocComment(_))
            | Ok(PklToken::LineComment(_))
            | Ok(PklToken::MultilineComment(_)) => Ok(parse_value(lexer)?),

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

/* ANCHOR: object */
/// Parse a token stream into a Pkl object.
fn parse_object<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> Result<PklValue<'a>> {
    let start = lexer.span().start;
    let mut hashmap = HashMap::new();
    let mut is_newline = true;

    loop {
        if let Some(token) = lexer.next() {
            match token {
                Ok(PklToken::Identifier(id)) | Ok(PklToken::IllegalIdentifier(id)) => {
                    if !is_newline {
                        return Err((
                            "unexpected token here (context: object), expected newline or comma"
                                .to_owned(),
                            lexer.span(),
                        ));
                    }

                    let value = parse_const_value(lexer)?;

                    if let PklValue::Object(_, _) = &value {
                        is_newline = true
                    } else {
                        is_newline = false
                    }

                    hashmap.insert(id, value);
                    continue;
                }
                Ok(PklToken::NewLine) | Ok(PklToken::Comma) => {
                    is_newline = true;
                    continue;
                }
                Ok(PklToken::Space) => continue,
                Ok(PklToken::CloseBrace) => break,

                Err(e) => return Err((e.to_string(), lexer.span())),
                _ => {
                    return Err((
                        "unexpected token here (context: object)".to_owned(),
                        lexer.span(),
                    ))
                }
            }
        } else {
            return Err(("Missing object close brace".to_owned(), lexer.span()));
        }
    }

    let end = lexer.span().end;
    Ok(PklValue::Object(hashmap, start..end))
}
/* ANCHOR_END: object */

/* ANCHOR: const */
/// Parse a token stream into a Pkl const Statement.
fn parse_const<'a>(lexer: &mut Lexer<'a, PklToken<'a>>, name: &'a str) -> Result<PklStatement<'a>> {
    let start = lexer.span().start;
    let value = parse_const_value(lexer)?;
    let end = lexer.span().end;

    Ok(PklStatement::Constant(name, value, start..end))
}
/* ANCHOR_END: const */

/* ANCHOR: const_value */
/// Parse a token stream into a Pkl Value after an identifier.
fn parse_const_value<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> Result<PklValue<'a>> {
    if let Some(token) = lexer.next() {
        match token {
            Ok(PklToken::EqualSign) => {
                let value = parse_value(lexer)?;

                Ok(value)
            }
            Ok(PklToken::OpenBrace) => {
                let value = parse_object(lexer)?;

                Ok(value)
            }
            Ok(PklToken::Space) => Ok(parse_const_value(lexer)?),
            Ok(PklToken::NewLine) => Ok(parse_const_value(lexer)?),
            Ok(PklToken::DocComment(_))
            | Ok(PklToken::LineComment(_))
            | Ok(PklToken::MultilineComment(_)) => Ok(parse_const_value(lexer)?),

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
/* ANCHOR_END: const_value */
