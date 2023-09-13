use std::rc::Rc;

use anyhow::{anyhow, bail};

use crate::expressions::{Expr, SExpr};
use crate::token2expr::parse_token;
use crate::tokenizer::Token;

/*** RECURSION WARNING
The following function recurses; this is by default limited to a stack of 128.
This could be fixed with an explicit stack, which was my first attempt.
The borrow checker did not like it. So, need to learn how build a tree w/out recursion in rust.
This _could_ be a use case for a bit of `unsafe`? But I think it's possible w/out it.
 */

/// Turn a stream of tokens into an S-expression
pub fn parse_tokens<'a>(
    token_iter: &mut impl Iterator<Item = &'a Token>,
) -> anyhow::Result<SExpr> {
    /**********************\
    |* Handle n=0 and n=1 *|
    \**********************/

    /* *** First token must be an open parentheses *** */
    let first_token = token_iter
        .next()
        .ok_or(anyhow!("No tokens"))?;

    // Surely there's a nicer way to write this?
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
) -> anyhow::Result<SExpr> {
    let mut sexpr = SExpr::new();

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
                sexpr.push(Rc::new(Expr::SExpr(sub_expr)));
            },
            token => {
                let next_expr = parse_token(token)?;
                sexpr.push(Rc::new(next_expr));
            },
        }
    }

    Ok(sexpr)
}
