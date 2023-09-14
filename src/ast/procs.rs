use std::rc::Rc;

use thiserror::Error;

use super::expressions::Expr;
use crate::scope::Scope;

#[derive(Error, Debug)]
pub enum ProcError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),

    #[error("type error: expected {expected}, got {actual}")]
    Type { expected: String, actual: String },

    #[error(
        "Function {name} takes {arity} arguments but got {num_args_provided}"
    )]
    Arity {
        name: String,
        arity: usize,
        num_args_provided: usize,
    },
}

pub type ProcResult = Result<Rc<Expr>, ProcError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Arity {
    Fixed(usize),
    Variadic,
}

impl Arity {
    pub fn check(&self, name: &str, n_args: usize) -> Result<(), ProcError> {
        if let &Arity::Fixed(arity) = self {
            if n_args != arity {
                return Err(ProcError::Arity {
                    name: name.to_string(),
                    arity,
                    num_args_provided: n_args,
                });
            }
        };

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proc {
    pub name: String,
    pub arity: Arity,
    pub eval: fn(&[Rc<Expr>]) -> ProcResult,
}

impl Proc {
    pub fn evaluate(&self, args: &[Rc<Expr>]) -> ProcResult {
        self.arity
            .check(&self.name, args.len())?;
        (self.eval)(args)
    }
}

/// Unlike procs, special form arguments are not evaluated, and
/// they can access and manipulate the scope
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Special {
    pub name: String,
    pub arity: Arity,
    pub eval: fn(&[Rc<Expr>], &mut Scope) -> ProcResult,
}

impl Special {
    pub fn evaluate(&self, args: &[Rc<Expr>], scope: &mut Scope) -> ProcResult {
        self.arity
            .check(&self.name, args.len())?;
        (self.eval)(args, scope)
    }
}
