/// Parses an identifier from the input stream.
///
/// This macro generates code to parse an identifier token from the given lexer.
/// It can be called with just a lexer for default error messages, or with a lexer
/// and custom error messages.
///
/// # Arguments
///
/// * `$lexer` - An expression that evaluates to a mutable reference to a Lexer.
/// * `$default_unexpected` - (Optional) A custom error message for unexpected tokens.
/// * `$eof_error` - (Optional) A custom error message for unexpected end of file.
///
/// # Returns
///
/// Returns a `Result` containing either:
/// * `Ok((&str, Range<usize>))` - A tuple with the identifier string and its span.
/// * `Err((String, Range<usize>))` - A tuple with an error message and the error span.
///
/// # Examples
///
/// ```
/// // Using default error messages
/// let result = parse_identifier!(lexer);
///
/// // Using custom error messages
/// let result = parse_identifier!(
///     lexer,
///     "Custom unexpected token error",
///     "Custom end of file error"
/// );
/// ```
#[macro_export]
macro_rules! parse_identifier {
    // Pattern 1: Just the lexer
    ($lexer:expr) => {
        parse_identifier!(
            $lexer,
            "unexpected token here, expected an identifier",
            "Expected identifier"
        )
    };
    ($lexer:expr, $default_unexpected:expr) => {
        parse_identifier!($lexer, $default_unexpected, "Expected identifier")
    };
    // Pattern 2: Lexer with custom error messages
    ($lexer:expr, $default_unexpected:expr, $eof_error:expr) => {{
        use crate::lexer::PklToken;
        let start = $lexer.span().start;
        while let Some(token) = $lexer.next() {
            match token {
                Ok(PklToken::Identifier(id)) | Ok(PklToken::IllegalIdentifier(id)) => {
                    return Ok((id, start..$lexer.span().end))
                }
                Ok(PklToken::NewLine) | Ok(PklToken::Space) => {
                    // Skip spaces and newlines
                }
                Err(e) => {
                    return Err((e.to_string(), $lexer.span()));
                }
                _ => {
                    return Err(($default_unexpected.to_owned(), $lexer.span()));
                }
            }
        }
        Err(($eof_error.to_owned(), $lexer.span()))
    }};
}

/// Parses a string from the input stream.
///
/// This macro generates code to parse a string token from the given lexer.
/// It can be called with just a lexer for default error messages, or with a lexer
/// and custom error messages.
///
/// # Arguments
///
/// * `$lexer` - An expression that evaluates to a mutable reference to a Lexer.
/// * `$default_unexpected` - (Optional) A custom error message for unexpected tokens.
/// * `$eof_error` - (Optional) A custom error message for unexpected end of file.
///
/// # Returns
///
/// Returns a `PklResult` containing either:
/// * `Ok((&str, Range<usize>))` - A tuple with the string content and its span.
/// * `Err((String, Range<usize>))` - A tuple with an error message and the error span.
///
/// # Examples
///
/// ```
/// // Using default error messages
/// let result = parse_string!(lexer);
///
/// // Using custom error messages
/// let result = parse_string!(
///     lexer,
///     "Custom unexpected token error",
///     "Custom end of file error"
/// );
/// ```
#[macro_export]
macro_rules! parse_string {
    // Pattern 1: Just the lexer
    ($lexer:expr) => {
        parse_string!(
            $lexer,
            "unexpected token here, expected a string",
            "Expected string"
        )
    };
    ($lexer:expr, $default_unexpected:expr) => {
        parse_identifier!($lexer, $default_unexpected, "Expected string")
    };
    // Pattern 2: Lexer with custom error messages
    ($lexer:expr, $default_unexpected:expr, $eof_error:expr) => {{
        let start = $lexer.span().start;
        while let Some(token) = $lexer.next() {
            match token {
                Ok(PklToken::String(s)) => return Ok((s, start..$lexer.span().end)),
                Ok(PklToken::NewLine) | Ok(PklToken::Space) => {
                    // Skip spaces and newlines
                }
                Err(e) => {
                    return Err((e.to_string(), $lexer.span()));
                }
                _ => {
                    return Err(($default_unexpected.to_owned(), $lexer.span()));
                }
            }
        }
        Err(($eof_error.to_owned(), $lexer.span()))
    }};
}
