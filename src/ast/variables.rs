use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::ast::Expr;

/// `Vars` are our AST nodes, represented as a pointer to an
/// expression

#[derive(Debug, PartialEq, Clone)]
pub struct Var(Rc<Expr>);

impl Var {
    pub fn new(expr: Expr) -> Self {
        Var(Rc::new(expr))
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // just delegate to the actual expr for now
        self.0.fmt(f)
    }
}

impl Deref for Var {
    type Target = Rc<Expr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Var {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Expr> for Var {
    fn from(value: Expr) -> Self {
        Var::new(value)
    }
}
