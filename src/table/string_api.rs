use crate::{PklResult, PklValue};
use base64::prelude::*;
use std::ops::Range;

/// Based on v0.26.0
pub fn match_string_props_api<'a, 'b>(
    s: &'a str,
    property: &'a str,
    range: Range<usize>,
) -> PklResult<PklValue<'b>> {
    match property {
        "length" => return Ok(PklValue::Int(s.len() as i64)),
        "lastIndex" => {
            return Ok(PklValue::Int({
                if s.len() == 0 {
                    -1
                } else {
                    (s.len() - 1) as i64
                }
            }))
        }
        "isEmpty" => return Ok(PklValue::Bool(s.len() == 0)),
        "isBlank" => return Ok(PklValue::Bool(s.trim().len() == 0)),
        "isRegex" => {
            return Err((
                "isRegex String API method not yet supported".to_owned(),
                range,
            ))
        }
        "md5" => return Err(("md5 String API method not yet supported".to_owned(), range)),
        "sha1" => return Err(("sha1 String API method not yet supported".to_owned(), range)),
        "sha256" => {
            return Err((
                "sha256 String API method not yet supported".to_owned(),
                range,
            ))
        }
        "sha256Int" => {
            return Err((
                "sha256Int String API method not yet supported".to_owned(),
                range,
            ))
        }
        "base64" => return Ok(PklValue::String(BASE64_STANDARD.encode(s))),
        "base64Decoded" => {
            let buf: Vec<u8> = BASE64_STANDARD.decode(s).map_err(|e| {
                (
                    format!("Failed to decode base64: {}", e.to_string()),
                    range.to_owned(),
                )
            })?;

            let s = std::str::from_utf8(&buf)
                .map_err(|e| (format!("Invalid UTF-8 sequence: {}", e.to_string()), range))?;

            return Ok(PklValue::String(s.to_owned()));
        }
        "chars" => {
            let chars = s
                .chars()
                .into_iter()
                .map(|c| PklValue::Char(c))
                .collect::<Vec<_>>();

            return Ok(PklValue::List(chars));
        }
        "codePoints" => {
            // would be better to have the Int as an u32
            let codepoints = s
                .chars()
                .into_iter()
                .map(|c| PklValue::Int(c as i64))
                .collect::<Vec<_>>();

            return Ok(PklValue::List(codepoints));
        }
        _ => {
            return Err((
                format!("String does not possess {} property", property),
                range,
            ))
        }
    }
}
