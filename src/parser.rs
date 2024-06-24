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

    /// ### An object amending another object:
    /// - First comes the name of the amended object,
    /// - Then the additional values
    /// - Finally the range
    AmendingObject(&'a str, Box<PklValue<'a>>, Range<usize>),
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
            | PklValue::AmendingObject(_, _, rng)
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

    while let Some(token) = lexer.next() {
        match token {
            Ok(PklToken::Identifier(id)) | Ok(PklToken::IllegalIdentifier(id)) => {
                if !is_newline {
                    return Err((
                        "unexpected token here (context: global), expected newline".to_owned(),
                        lexer.span(),
                    ));
                }
                let statement = parse_const(lexer, id)?;
                statements.push(statement);
                is_newline = false;
            }
            Ok(PklToken::Space)
            | Ok(PklToken::DocComment(_))
            | Ok(PklToken::LineComment(_))
            | Ok(PklToken::MultilineComment(_)) => {
                // Skip spaces and comments
                continue;
            }
            Ok(PklToken::NewLine) => {
                is_newline = true;
                continue;
            }
            Err(e) => return Err((e.to_string(), lexer.span())),
            _ => {
                return Err((
                    "unexpected token here (context: statement)".to_owned(),
                    lexer.span(),
                ))
            }
        }
    }

    Ok(statements)
}
/* ANCHOR_END: statement */

/* ANCHOR: value */
/// Parse a token stream into a Pkl value.
fn parse_value<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> Result<PklValue<'a>> {
    loop {
        match lexer.next() {
            Some(Ok(PklToken::Bool(b))) => return Ok(PklValue::Bool(b, lexer.span())),
            Some(Ok(PklToken::Int(i)))
            | Some(Ok(PklToken::OctalInt(i)))
            | Some(Ok(PklToken::HexInt(i)))
            | Some(Ok(PklToken::BinaryInt(i))) => return Ok(PklValue::Int(i, lexer.span())),
            Some(Ok(PklToken::Float(f))) => return Ok(PklValue::Float(f, lexer.span())),
            Some(Ok(PklToken::String(s))) => return Ok(PklValue::String(s, lexer.span())),
            Some(Ok(PklToken::MultiLineString(s))) => {
                return Ok(PklValue::MultiLineString(s, lexer.span()))
            }
            Some(Ok(PklToken::OpenParen)) => return parse_amended_object(lexer),
            Some(Ok(PklToken::Space))
            | Some(Ok(PklToken::NewLine))
            | Some(Ok(PklToken::DocComment(_)))
            | Some(Ok(PklToken::LineComment(_)))
            | Some(Ok(PklToken::MultilineComment(_))) => continue,
            Some(Err(e)) => return Err((e.to_string(), lexer.span())),
            Some(_) => {
                return Err((
                    "unexpected token here (context: value)".to_owned(),
                    lexer.span(),
                ))
            }
            None => return Err(("empty values are not allowed".to_owned(), lexer.span())),
        }
    }
}
/* ANCHOR_END: value */

/* ANCHOR: object */
/// Parse a token stream into a Pkl object.
fn parse_object<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> Result<PklValue<'a>> {
    let start = lexer.span().start;
    let mut hashmap = HashMap::new();
    let mut is_newline = true;

    while let Some(token) = lexer.next() {
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

                is_newline = matches!(value, PklValue::Object(_, _));

                hashmap.insert(id, value);
            }
            Ok(PklToken::NewLine) | Ok(PklToken::Comma) => {
                is_newline = true;
            }
            Ok(PklToken::Space) => {
                // Skip spaces
            }
            Ok(PklToken::CloseBrace) => {
                let end = lexer.span().end;
                return Ok(PklValue::Object(hashmap, start..end));
            }
            Err(e) => {
                return Err((e.to_string(), lexer.span()));
            }
            _ => {
                return Err((
                    "unexpected token here (context: object)".to_owned(),
                    lexer.span(),
                ));
            }
        }
    }

    Err(("Missing object close brace".to_owned(), lexer.span()))
}
/* ANCHOR_END: object */

fn parse_amended_object<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> Result<PklValue<'a>> {
    let start = lexer.span().start;

    let amended_object_name = match lexer.next() {
        Some(Ok(PklToken::Identifier(id))) | Some(Ok(PklToken::IllegalIdentifier(id))) => {
            match lexer.next() {
                Some(Ok(PklToken::CloseParen)) => id,
                Some(Err(e)) => return Err((e.to_string(), lexer.span())),
                _ => {
                    return Err((
                        "expected close parenthesis (context: amended_object)".to_owned(),
                        lexer.span(),
                    ))
                }
            }
        }
        Some(Err(e)) => return Err((e.to_string(), lexer.span())),
        _ => {
            return Err((
                "expected identifier here (context: amended_object)".to_owned(),
                lexer.span(),
            ))
        }
    };

    while let Some(token) = lexer.next() {
        match token {
            Ok(PklToken::Space) | Ok(PklToken::NewLine) => continue,
            Ok(PklToken::OpenBrace) => {
                let object = parse_object(lexer)?;
                let end = lexer.span().end;

                return Ok(PklValue::AmendingObject(
                    amended_object_name,
                    Box::new(object),
                    start..end,
                ));
            }
            Err(e) => return Err((e.to_string(), lexer.span())),
            _ => {
                return Err((
                    "expected open brace here (context: amended_object)".to_owned(),
                    lexer.span(),
                ))
            }
        }
    }

    Err((
        "expected open brace (context: amended_object)".to_owned(),
        lexer.span(),
    ))
}

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
    loop {
        match lexer.next() {
            Some(Ok(PklToken::EqualSign)) => {
                return parse_value(lexer);
            }
            Some(Ok(PklToken::OpenBrace)) => {
                return parse_object(lexer);
            }
            Some(Ok(PklToken::Space))
            | Some(Ok(PklToken::NewLine))
            | Some(Ok(PklToken::DocComment(_)))
            | Some(Ok(PklToken::LineComment(_)))
            | Some(Ok(PklToken::MultilineComment(_))) => {
                // Continue the loop to process the next token
                continue;
            }
            Some(Err(e)) => {
                return Err((e.to_string(), lexer.span()));
            }
            Some(_) => {
                return Err((
                    "unexpected token here (context: constant)".to_owned(),
                    lexer.span(),
                ));
            }
            None => {
                return Err(("Expected '='".to_owned(), lexer.span()));
            }
        }
    }
}
/* ANCHOR_END: const_value */
