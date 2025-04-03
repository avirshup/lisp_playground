use anyhow::{Result, anyhow, bail};

use super::token_handlers::parse_token;
use super::tokenizer::{Token, tokenize};
use crate::ast::{Expr, OwnedSExpr, Value, Var};

/// turn text into an s-expression
pub fn parse_text(s: &str) -> Result<OwnedSExpr> {
    let tokens = tokenize(s);
    parse_tokens(&mut tokens.iter())
}

/// Turn a stream of tokens into an S-expression
///
/// WARNING: This function recurses! By default limited to a stack of 128.
/// This could be fixed with an explicit stack, which was my first attempt.
/// The borrow checker did not like it. So, need to learn how build a tree w/out
/// recursion in rust. This _could_ be a use case for a bit of `unsafe`? But I
/// think it's possible w/out it.
pub fn parse_tokens<'a>(
    token_iter: &mut impl Iterator<Item = &'a Token>,
) -> Result<OwnedSExpr> {
    /**********************\
    |* Handle n=0 and n=1 *|
    \**********************/

    /* *** First token must be an open parentheses *** */
    let first_token = token_iter
        .next()
        .ok_or(anyhow!("No tokens"))?;

    let Token::ParenStart = first_token else {
        bail!("Expression should begin with '(', but got {first_token:#?}")
    };

    /* ** Build the root S-expression ** */
    let root = build_sexpr(token_iter);

    // ensure tokens were exhausted
    // Surely there's a nicer way to write this?
    if root.is_ok() {
        if let Some(token) = token_iter.next() {
            bail!(
                "S-expression is complete, but tokens remain ({token:#?}). \
                 Unmatched closing parentheses?"
            )
        }
    }

    root
}

/// Build the s-expression from tokens
/// Will build nested s-expressions via recursion
fn build_sexpr<'a>(
    token_iter: &mut impl Iterator<Item = &'a Token>,
) -> Result<OwnedSExpr> {
    let mut sexpr = OwnedSExpr::new();

    loop {
        let token = token_iter.next().ok_or(anyhow!(
            "Token stream ended before S-Expression was complete"
        ))?;

        // add to the current s-expression as indicated via the token
        match token {
            Token::ParenEnd => {
                break;
            },
            Token::ParenStart => {
                let sub_expr = build_sexpr(token_iter)?;
                sexpr.push(Var::new(Expr::SExpr(sub_expr)));
            },
            Token::Dash => {
                let next_expr = token_iter
                    .next()
                    .ok_or(anyhow!(
                        "Token stream ended after negative sign"
                    ))
                    .and_then(parse_token)
                    .and_then(try_negate)?;
                sexpr.push(Var::new(next_expr));
            },

            token => {
                let next_expr = parse_token(token)?;
                sexpr.push(Var::new(next_expr));
            },
        }
    }

    Ok(sexpr)
}

fn try_negate(expr: Expr) -> Result<Expr> {
    match expr {
        Expr::Value(Value::Int(n)) => Ok(Value::Int(-n).into()),
        Expr::Value(Value::Float(f)) => Ok(Value::Float(-f).into()),
        other => Err(anyhow!("Can't negate expression {other:#?}")),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn do_literal_test(input: &str, expected: Value) {
        let wrapped = format!("({input})");
        let result = parse_text(&wrapped).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            *result.first().unwrap().as_ref(),
            Expr::Value(expected)
        );
    }

    #[test]
    fn test_parse_ints() {
        do_literal_test("0", Value::Int(0));
        do_literal_test("10", Value::Int(10));
        do_literal_test("00103", Value::Int(103));
    }

    #[test]
    fn test_parse_floats() {
        do_literal_test("0.", Value::Float(0.));
        do_literal_test("82.7110", Value::Float(82.7110));
        do_literal_test("010.", Value::Float(10.));
        do_literal_test("12e3", Value::Float(12000.));
    }

    #[test]
    fn test_parse_chars() {
        do_literal_test("c'0'", Value::Char('0'));
        do_literal_test("c\"👋\"", Value::Char('👋'));
        do_literal_test("c\"µ\"", Value::Char('µ'));
    }

    #[test]
    fn test_parse_negative_numbers() {
        do_literal_test("-1", Value::Int(-1));
        do_literal_test("-0", Value::Int(-0));
        do_literal_test("- 0010", Value::Int(-10));

        do_literal_test("-  0.", Value::Float(-0.));
        do_literal_test("- 82.7110", Value::Float(-82.7110));
        do_literal_test("-010.", Value::Float(-10.));
        do_literal_test("- 12e3", Value::Float(-12000.));
    }
}
