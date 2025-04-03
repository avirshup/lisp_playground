use std::fmt::{Display, Formatter};

use itertools::Itertools;

use super::{EvalError, Function, Mapping, SpecialForm, Value};
use crate::InternalError;
use crate::ast::variables::Var;

/// An S-expression is a slice of Vars
/// This will always show up in the form &SExpr
pub type SExpr = [Var];

/// Like SExpr, but owned
pub type OwnedSExpr = Vec<Var>;

/// Exprs are immutable value-type building blocks
#[derive(Debug, PartialEq)]
pub enum Expr {
    SExpr(OwnedSExpr),
    Function(Function),
    Special(SpecialForm),
    Symbol(String),
    Value(Value),
    Record(Mapping),
    Keyword(String),
}

impl Expr {
    pub fn empty() -> Self {
        Expr::SExpr(Vec::new())
    }

    pub fn type_str(&self) -> &'static str {
        match self {
            Expr::SExpr(_) => "S-expression",
            Expr::Symbol(_) => "Symbol",
            Expr::Value(_) => "Value",
            Expr::Keyword(_) => "Keyword",
            Expr::Function(_) => "Function",
            Expr::Special(_) => "SpecialForm",
            Expr::Record(_) => "Record",
        }
    }

    pub fn expect_symbol(&self) -> Result<&str, EvalError> {
        if let Expr::Symbol(name) = self {
            Ok(name)
        } else {
            Err(EvalError::Syntax {
                expected: "Symbol".to_string(),
                actual: self.type_str().to_string(),
            })
        }
    }

    pub fn expect_keyword(&self) -> Result<&str, EvalError> {
        if let Expr::Keyword(name) = self {
            Ok(name)
        } else {
            Err(EvalError::Syntax {
                expected: "Keyword".to_string(),
                actual: self.type_str().to_string(),
            })
        }
    }

    pub fn expect_fn(&self) -> Result<&Function, EvalError> {
        match self {
            Expr::Function(lisp_fn) => Ok(lisp_fn),
            _other => {
                Err(EvalError::Syntax {
                    expected: "Function".to_string(),
                    actual: self.type_str().to_string(),
                })
            },
        }
    }

    pub fn expect_sexp(&self) -> Result<&SExpr, EvalError> {
        match self {
            Expr::SExpr(sexp) => Ok(sexp),
            _other => {
                Err(EvalError::Syntax {
                    expected: "S-expression".to_string(),
                    actual: self.type_str().to_string(),
                })
            },
        }
    }

    pub fn expect_sexp_with_len(&self, len: usize) -> Result<&SExpr, EvalError> {
        let sexp = self.expect_sexp()?;
        let actual_len = sexp.len();

        if actual_len != len {
            Err(EvalError::Syntax {
                expected: format!("S-expression w/ length {len}"),
                actual: format!("Length {len}"),
            })
        } else {
            Ok(sexp)
        }
    }

    pub fn expect_special(&self) -> Result<&SpecialForm, EvalError> {
        match self {
            Expr::Special(special) => Ok(special),
            _other => {
                Err(EvalError::Syntax {
                    expected: "Special".to_string(),
                    actual: self.type_str().to_string(),
                })
            },
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::SExpr(sexp) => write!(f, "{}", display_sexp(sexp)),
            Expr::Symbol(name) => write!(f, "#Symbol[{}]", name),
            Expr::Keyword(s) => write!(f, ":{}", s),
            Expr::Value(ctype) => ctype.fmt(f),
            Expr::Function(func) => func.fmt(f),
            Expr::Special(s) => s.fmt(f),
            Expr::Record(rec) => write!(f, "{}", display_record(rec)),
        }
    }
}

// TODO: this could go much faster by taking the `&mut Formatter` buffer
//   and writing into it directly
fn display_sexp(sexp: &SExpr) -> String {
    let items = sexp
        .iter()
        .map(|e| e.as_ref().to_string())
        .collect::<Vec<String>>()
        .join(" ");
    format!("( {items} )")
}

fn display_record(record: &Mapping) -> String {
    let formatter = record
        .iter()
        .format_with(", ", |(k, v), f| {
            f(&format_args!("  :{k}->{v}"))
        });
    format!("{{\n{}\n}}", formatter)
}

/***** CONVERSIONS ******* */
// (These are for programming convenience,
// not any sort of language semantics)

/*************************\
|* Exprs into Value types *|
\*************************/
impl<'a> TryFrom<&'a Expr> for &'a Value {
    type Error = InternalError;

    fn try_from(var: &'a Expr) -> Result<Self, InternalError> {
        if let Expr::Value(v) = var {
            Ok(v)
        } else {
            Err(InternalError::NotAValue {
                expression: format!("{}", var),
            })
        }
    }
}

/****************************\
|* Exprs from wrapped types *|
\****************************/
// Coercion sugar to make it easier to create exprs and vars
macro_rules! impl_raw_expr_conversions {
    ($($t:ty, $v:ident);* $(;)?) => {
        $(
            impl From<$t> for Expr {
                fn from(value: $t) -> Self {
                    Expr::$v(value)
                }
            }

            impl From<$t> for Var {
                fn from(value: $t) -> Self {
                    Var::new(Expr::$v(value))
                }
            }
        )*
    };
}

impl_raw_expr_conversions!(
    Value, Value;
    OwnedSExpr, SExpr;
    Function, Function;
    SpecialForm, Special;
);
