use anyhow::{bail, Result};

use super::expressions::CType::*;
use super::token2expr::parse_token;
use super::tokenizer::Token;

pub enum CType {
    Str(String),
    Char(char),
    Int(isize),
    Bytes(Vec<u8>),
    Float(f64),
    Bool(bool),
    Nil,
}

impl CType {
    /// Wrap a "bare" literal with an Expr::Lit
    pub fn expr(self) -> Expr {
        Expr::Lit(self)
    }
}

pub type SExpr = Vec<Expr>;

pub enum Expr {
    // recursive s-exprs
    SExpr(SExpr),
    Special(SExpr),

    // leaf nodes
    Symbol(String),
    Lit(CType),
    Keyword(String),
}

pub fn read_tokens(tokens: &[Token]) -> Result<SExpr> {
    /**********************\
    |* Handle n=0 and n=1 *|
    \**********************/
    let mut root: SExpr = Vec::new();

    if tokens.len() == 0 {
        return Ok(root);
    }
    if tokens.len() == 1 {
        // TODO implement this
        bail!("Couldn't evaluate single-token exepression")
    }

    /* *** First token must be an open parentheses *** */
    let mut token_iter = tokens.iter();
    if *token_iter.next().unwrap() != Token::ParenStart {
        bail!("Expression must begin with '('")
    }

    // this is the stack of active s-expressions
    let mut sexpr_stack: Vec<&SExpr> = vec![&root];

    for token in token_iter {
        // we're working on the s-expression on the bottom of the stack
        let mut current_sexpr = sexpr_stack.last_mut().unwrap();

        // add to the current s-expression as indicated via the token
        match token {
            Token::ParenStart => {
                let new_sexpr = Vec::new();
                sexpr_stack.push(&new_sexpr);
                current_sexpr.push(Expr::SExpr(new_sexpr));
            },
            Token::ParenEnd => {
                if sexpr_stack.len() == 0 {
                    bail!("Too many closing parentheses")
                }
                sexpr_stack.pop();
            },
            token => {
                let next_expr = parse_token(token)?;
                current_sexpr.push(next_expr);
            },
        }
    }

    if sexpr_stack.len() != 1 {
        bail!(
            "There were {} unclosed parentheses in this expression",
            sexpr_stack.len() - 1
        )
    }

    todo!()
}
