use std::fmt::{Display, Formatter};
use std::rc::Rc;

use super::procs::{Proc, Special};
use crate::ast::CType;

pub type SExpr = Vec<Rc<Expr>>;

/// Exprs are immutable value-type building blocks
#[derive(Debug, Clone, PartialEq)]
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

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::SExpr(sexp) => write!(f, "{}", display_sexp(sexp)),
            Expr::Symbol(name) => write!(f, "#Symbol[{}]", name),
            Expr::Lit(ctype) => ctype.fmt(f),
            Expr::Keyword(s) => write!(f, ":{}", s),
            Expr::Proc(p) => write!(f, "#Proc[{}]", p.name),
            Expr::Special(form) => write!(f, "#Form[{}]", form.name),
        }
    }
}

fn display_sexp(sexp: &SExpr) -> String {
    let items = sexp
        .iter()
        .map(|e| e.as_ref().to_string())
        .collect::<Vec<String>>()
        .join(" ");
    format!("( {items} )")
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
