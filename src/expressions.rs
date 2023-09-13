use std::rc::Rc;

use super::procs::{Proc, Special};
use crate::ctypes::CType;

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
    Proc(Proc),
    Special(Special),
}

impl Expr {
    pub fn empty() -> Self {
        Expr::SExpr(Vec::new())
    }
}

// Coercion sugar to make it easier to create exprs
macro_rules! impl_from_expr {
    ($($t:ty, $v:ident);* $(;)?) => {
        $(
            impl From<$t> for Expr {
                fn from(value: $t) -> Self {
                    Expr::$v(value)
                }
            }
        )*
    };
}

impl_from_expr!(
    CType, Lit;
    Proc, Proc;
    Special, Special;
    SExpr, SExpr;
);
