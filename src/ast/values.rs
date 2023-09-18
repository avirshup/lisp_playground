use std::fmt::{Display, Formatter};

use super::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Char(char),
    Int(isize),
    Bytes(Vec<u8>),
    Float(f64),
    Bool(bool), // are `true` / `false` symbols or lits? Right now a lit.
    Nil,
}

impl Value {
    /// For convenience - you usually want to wrap a "bare" literal
    /// with an Expr::Lit
    pub fn expr(self) -> Expr {
        Expr::Value(self)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(x) => x.fmt(f),
            Value::Char(x) => x.fmt(f),
            Value::Int(x) => x.fmt(f),
            Value::Bytes(_) => write!(f, "not implemented"),
            Value::Float(x) => x.fmt(f),
            Value::Bool(x) => x.fmt(f),
            Value::Nil => write!(f, "Nil"),
        }
    }
}
