use crate::lexer::PklToken;
use logos::{Lexer, Span};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut, Range},
};

pub type ParseError = (String, Span);
pub type PklResult<T> = std::result::Result<T, ParseError>;

/* ANCHOR: statements */
/// Represent any valid Pkl value.
#[derive(Debug, PartialEq, Clone)]
pub enum PklStatement<'a> {
    Constant(&'a str, PklExpr<'a>, Range<usize>),
}
/* ANCHOR_END: statements */

/* ANCHOR: expression */
/// Represent any valid Pkl expression.
#[derive(Debug, PartialEq, Clone)]
pub enum PklExpr<'a> {
    Identifier(&'a str, Range<usize>),
    Value(AstPklValue<'a>),
}

impl<'a> PklExpr<'a> {
    /// This function MUST be called only when we are sure `PklExpr` is a `AstPklValue`
    pub fn extract_value(self) -> AstPklValue<'a> {
        match self {
            Self::Value(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn span(&self) -> Range<usize> {
        match self {
            Self::Value(v) => v.span(),
            Self::Identifier(_, indexes) => indexes.to_owned(),
        }
    }
}
/* ANCHOR_END: expression */

impl<'a> From<AstPklValue<'a>> for PklExpr<'a> {
    fn from(value: AstPklValue<'a>) -> Self {
        PklExpr::Value(value)
    }
}
impl<'a> From<(&'a str, Range<usize>)> for PklExpr<'a> {
    fn from((value, indexes): (&'a str, Range<usize>)) -> Self {
        PklExpr::Identifier(value, indexes)
    }
}

type ExprHash<'a> = (HashMap<&'a str, PklExpr<'a>>, Range<usize>);

/* ANCHOR: values */
/// Represent any valid Pkl value.
#[derive(Debug, PartialEq, Clone)]
pub enum AstPklValue<'a> {
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
    Object(ExprHash<'a>),

    /// A Class instance.
    ClassInstance(&'a str, ExprHash<'a>, Range<usize>),

    /// ### An object amending another object:
    /// - First comes the name of the amended object,
    /// - Then the additional values
    /// - Finally the range
    ///
    /// **Corresponds to:**
    /// ```pkl
    /// x = (other_object) {
    ///     prop = "attribute"
    /// }
    /// ```
    AmendingObject(&'a str, ExprHash<'a>, Range<usize>),

    /// ### An amended object.
    /// Different from `AmendingObject`
    ///
    /// **Corresponds to:**
    /// ```pkl
    /// x = {
    ///    prop = "attribute"
    /// } {
    ///    other_prop = "other_attribute"
    /// }
    /// ```
    AmendedObject(Box<AstPklValue<'a>>, ExprHash<'a>, Range<usize>),
}
/* ANCHOR_END: values */

impl<'a> Deref for PklStatement<'a> {
    type Target = PklExpr<'a>;

    fn deref(&self) -> &Self::Target {
        match self {
            PklStatement::Constant(_, value, _) => value,
        }
    }
}
impl<'a> DerefMut for PklStatement<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            PklStatement::Constant(_, value, _) => value,
        }
    }
}
impl<'a> PklStatement<'a> {
    pub fn span(&self) -> Range<usize> {
        match self {
            PklStatement::Constant(_, _, rng) => rng.clone(),
        }
    }
}

impl<'a> From<ExprHash<'a>> for AstPklValue<'a> {
    fn from(value: ExprHash<'a>) -> Self {
        AstPklValue::Object(value)
    }
}
impl<'a> From<ExprHash<'a>> for PklExpr<'a> {
    fn from(value: ExprHash<'a>) -> Self {
        PklExpr::Value(value.into())
    }
}

impl<'a> AstPklValue<'a> {
    pub fn span(&self) -> Range<usize> {
        match self {
            AstPklValue::Int(_, rng)
            | AstPklValue::Bool(_, rng)
            | AstPklValue::Float(_, rng)
            | AstPklValue::Object((_, rng))
            | AstPklValue::AmendingObject(_, _, rng)
            | AstPklValue::AmendedObject(_, _, rng)
            | AstPklValue::ClassInstance(_, _, rng)
            | AstPklValue::String(_, rng)
            | AstPklValue::MultiLineString(_, rng) => rng.clone(),
        }
    }
}

/* ANCHOR: statement */
/// Parse a token stream into a Pkl statement.
pub fn parse_pkl<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> PklResult<Vec<PklStatement<'a>>> {
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
            Ok(PklToken::OpenBrace) => {
                if let Some(PklStatement::Constant(_, value, rng)) = statements.last_mut() {
                    match value {
                        PklExpr::Value(AstPklValue::Object((_, _)))
                        | PklExpr::Value(AstPklValue::AmendingObject(_, _, _))
                        | PklExpr::Value(AstPklValue::AmendedObject(_, _, _)) => {
                            let new_object = parse_object(lexer)?;
                            let start = rng.start;
                            let end = new_object.1.end;
                            *value = AstPklValue::AmendedObject(
                                Box::new(value.clone().extract_value()),
                                new_object,
                                start..end,
                            )
                            .into()
                        }
                        _ => {
                            return Err((
                                "unexpected token here (context: global)".to_owned(),
                                lexer.span(),
                            ))
                        }
                    }
                } else {
                    return Err((
                        "unexpected token here (context: global)".to_owned(),
                        lexer.span(),
                    ));
                }
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

/* ANCHOR: expression */
/// Parse a token stream into a Pkl expression.
fn parse_expr<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> PklResult<PklExpr<'a>> {
    loop {
        match lexer.next() {
            Some(Ok(PklToken::Bool(b))) => return Ok(AstPklValue::Bool(b, lexer.span()).into()),
            Some(Ok(PklToken::Identifier(id))) | Some(Ok(PklToken::IllegalIdentifier(id))) => {
                return Ok(PklExpr::Identifier(id, lexer.span()))
            }
            Some(Ok(PklToken::New)) => return parse_class_instance(lexer),

            Some(Ok(PklToken::Int(i)))
            | Some(Ok(PklToken::OctalInt(i)))
            | Some(Ok(PklToken::HexInt(i)))
            | Some(Ok(PklToken::BinaryInt(i))) => {
                return Ok(AstPklValue::Int(i, lexer.span()).into())
            }
            Some(Ok(PklToken::Float(f))) => return Ok(AstPklValue::Float(f, lexer.span()).into()),
            Some(Ok(PklToken::String(s))) => return Ok(AstPklValue::String(s, lexer.span()).into()),
            Some(Ok(PklToken::MultiLineString(s))) => {
                return Ok(AstPklValue::MultiLineString(s, lexer.span()).into())
            }
            Some(Ok(PklToken::OpenParen)) => return Ok(parse_amended_object(lexer)?.into()),
            Some(Ok(PklToken::Space))
            | Some(Ok(PklToken::NewLine))
            | Some(Ok(PklToken::DocComment(_)))
            | Some(Ok(PklToken::LineComment(_)))
            | Some(Ok(PklToken::MultilineComment(_))) => continue,
            Some(Err(e)) => return Err((e.to_string(), lexer.span())),
            Some(_) => {
                return Err((
                    "unexpected token here (context: expression)".to_owned(),
                    lexer.span(),
                ))
            }
            None => return Err(("empty expressions are not allowed".to_owned(), lexer.span())),
        }
    }
}
/* ANCHOR_END: expression */

/* ANCHOR: object */
/// Parse a token stream into a Pkl object.
fn parse_object<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> PklResult<ExprHash<'a>> {
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

                let value = parse_const_expr(lexer)?;

                is_newline = matches!(value, PklExpr::Value(AstPklValue::Object((_, _))));

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
                return Ok((hashmap, start..end));
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

fn parse_amended_object<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> PklResult<AstPklValue<'a>> {
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

                return Ok(AstPklValue::AmendingObject(
                    amended_object_name,
                    object,
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
fn parse_const<'a>(
    lexer: &mut Lexer<'a, PklToken<'a>>,
    name: &'a str,
) -> PklResult<PklStatement<'a>> {
    let start = lexer.span().start;
    let value = parse_const_expr(lexer)?;
    let end = lexer.span().end;

    Ok(PklStatement::Constant(name, value, start..end))
}
/* ANCHOR_END: const */

/* ANCHOR: const_expr */
/// Parse a token stream into a Pkl Expr after an identifier.
fn parse_const_expr<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> PklResult<PklExpr<'a>> {
    loop {
        match lexer.next() {
            Some(Ok(PklToken::EqualSign)) => {
                return parse_expr(lexer);
            }
            Some(Ok(PklToken::OpenBrace)) => {
                return Ok(parse_object(lexer)?.into());
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
/* ANCHOR_END: const_expr */

fn parse_class_instance<'a>(lexer: &mut Lexer<'a, PklToken<'a>>) -> PklResult<PklExpr<'a>> {
    let start = lexer.span().start;

    let class_name = loop {
        match lexer.next() {
            Some(Ok(PklToken::Identifier(id))) | Some(Ok(PklToken::IllegalIdentifier(id))) => {
                break id
            }
            Some(Ok(PklToken::Space))
            | Some(Ok(PklToken::NewLine))
            | Some(Ok(PklToken::DocComment(_)))
            | Some(Ok(PklToken::LineComment(_)))
            | Some(Ok(PklToken::MultilineComment(_))) => continue,
            Some(Err(e)) => return Err((e.to_string(), lexer.span())),
            Some(_) => {
                return Err((
                    "unexpected token here (context: class_instance), expected identifier"
                        .to_owned(),
                    lexer.span(),
                ));
            }
            None => return Err(("Expected identifier".to_owned(), lexer.span())),
        }
    };

    loop {
        match lexer.next() {
            Some(Ok(PklToken::OpenBrace)) => {
                return Ok(AstPklValue::ClassInstance(
                    class_name,
                    parse_object(lexer)?,
                    start..lexer.span().end,
                )
                .into());
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
