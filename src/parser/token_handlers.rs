use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use litrs::Literal;
use regex::Regex;

use super::tokenizer::{Quote, Token};
use crate::ast::{Expr, Value};

/// Parses non-paren tokens
pub fn parse_token(t: &Token) -> Result<Expr> {
    match t {
        Token::Word(s) => {
            parse_literal(s)
                .map(Expr::Value)
                .or_else(|_| parse_identifier(s))
        },

        Token::StringLit(q) => parse_quote(q).map(Expr::Value),

        _ => Err(anyhow!("Unhandled token type: {:#?}", t)),
    }
}

/// Try to parse a word as a literal, more or less the same way as rust does
fn parse_literal(s: &str) -> Result<Value> {
    Literal::parse(s)
        .map_err(anyhow::Error::from)
        .and_then(check_suffix)
        .and_then(|r| {
            match r {
                // TODO: should bools be literals or just symbols?
                Literal::Bool(_) => Ok(Value::Bool(s.parse()?)),

                Literal::Integer(_) => Ok(Value::Int(s.parse()?)),

                Literal::Float(lit) => {
                    Ok(Value::Float(lit.number_part().parse()?))
                },

                // NOTE: there is no token for this yet
                Literal::Char(_) => {
                    Ok(Value::Char(s.chars().next().ok_or(
                        anyhow!("No character in string? '{s}'"),
                    )?))
                },

                lit => {
                    Err(anyhow!(
                        "eaten by the worms, and weird fishes: '{lit:#?}'"
                    ))
                },
            }
        })
}

/// Parse a word that must be keyword or a symbol
/// Or `nil`, which is probably incorrectly treated as a literal?
/// Must only be called after ensuring that the word is not a literal.
fn parse_identifier(s: &str) -> Result<Expr> {
    if SYMBOL_RE.is_match(s) {
        if s.starts_with(':') {
            // it's a keyword
            Ok(Expr::Keyword(s.to_string()))
        } else if s.to_lowercase() == "nil" {
            // it's probably nil
            if s == "nil" {
                Ok(Expr::Value(Value::Nil))
            } else {
                Err(anyhow!(
                    "`nil` must be lowercase (got '{s}')"
                ))
            }
        } else {
            // it's a symbol
            Ok(Expr::Symbol(s.to_string()))
        }
    } else {
        // it's invalid
        Err(anyhow!("'{}' is not a keyword or symbol", s))
    }
}

/// Parse a quoted string. The current treatment should be nearly identical to
/// rust, except that:
/// A) single-quotes are treated as equivalent to double-quotes, and
/// B) characters are denoted by the sigil c and must have len 1
fn parse_quote(quote: &Quote) -> Result<Value> {
    let lits = format!("{}\"{}\"", quote.sigil, quote.content);
    if quote.sigil == "c" {
        let chars: Vec<char> = quote.content.chars().collect();
        if chars.len() == 1 {
            Ok(Value::Char(*chars.first().unwrap()))
        } else {
            Err(anyhow!(
                "Quote had sigil `c` but not exactly one character: {quote:#?}"
            ))
        }
    } else {
        Literal::parse(lits)
            .map_err(anyhow::Error::from)
            .and_then(|r| {
                match r {
                    Literal::String(sl) => {
                        Ok(Value::Str(sl.into_value().to_string()))
                    },
                    Literal::ByteString(bl) => {
                        Ok(Value::Bytes(bl.into_value().to_vec()))
                    },
                    _ => Err(anyhow!("Failed to parse quote: {quote:#?}")),
                }
            })
    }
}

/***********\
|* Helpers *|
\***********/
lazy_static! {
    static ref SYMBOL_RE: Regex =
        Regex::new(r"^:?[a-zA-Z*+!\-_?][a-zA-Z0-9*+!\-_?]*$").unwrap();
}

/// Ensure literal doesn't have a suffix
/// E.g., "15" is ok, "15u32" is not, because u32 is a rust thing)
fn check_suffix(lit: Literal<&str>) -> Result<Literal<&str>> {
    {
        if lit.suffix() == "" {
            Ok(lit)
        } else {
            Err(anyhow!(
                "Forbidden suffix '{}' on literal '{}'",
                lit.suffix(),
                lit.to_string()
            ))
        }
    }
}
