use std::rc::Rc;

use anyhow::{anyhow, bail, Result};

use super::procs::{Proc, Special};
use super::token2expr::parse_token;
use super::tokenizer::Token;

#[derive(Debug, Clone)]
pub enum CType {
    Str(String),
    Char(char),
    Int(isize),
    Bytes(Vec<u8>),
    Float(f64),
    Bool(bool), // are `true` / `false` symbols or lits? Right now a lit.
    Nil,
}

impl CType {
    /// For convenience - you usually want to wrap a "bare" literal
    /// with an Expr::Lit
    pub fn expr(self) -> Expr {
        Expr::Lit(self)
    }
}

pub type SExpr = Vec<Rc<Expr>>;

/// Exprs are immutable value-type building blocks
#[derive(Debug, Clone)]
pub enum Expr {
    // recursive s-exprs
    SExpr(SExpr),

    // leaf nodes
    Symbol(String),
    Lit(CType),
    Keyword(String),
    Proc(&'static Proc),
    Special(&'static Special),
}

/*** RECURSION WARNING
The following function recurses; this is by default limited to a stack of 128.
This could be fixed with an explicit stack, which was my first attempt.
The borrow checker did not like it. So, need to learn how build a tree w/out recursion in rust.
This _could_ be a use case for a bit of `unsafe`? But I think it's possible w/out it.
 */

/// Build the s-expression from tokens
/// Will build nested s-expressions via recursion
fn build_sexpr<'a>(
    token_iter: &mut impl Iterator<Item = &'a Token>,
) -> Result<SExpr> {
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

pub fn read_tokens<'a>(
    token_iter: &mut impl Iterator<Item = &'a Token>,
) -> Result<SExpr> {
    /**********************\
    |* Handle n=0 and n=1 *|
    \**********************/

    /* *** First token must be an open parentheses *** */
    let first_token = token_iter
        .next()
        .ok_or(anyhow!("No tokens"))?;

    // Surely there's a nicer way to write this
    match first_token {
        Token::ParenStart => (),
        other => bail!("Expression should begin with '(', but got {other:#?}"),
    }

    /* ** Build the root S-expression ** */
    let root = build_sexpr(token_iter);

    // ensure tokens were exhausted
    // Surely there's a nicer way to write this
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
