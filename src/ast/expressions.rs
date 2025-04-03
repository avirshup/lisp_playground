use std::fmt::{format, Display, Formatter};
use std::rc::Rc;

use itertools::Itertools;

use super::{EvalError, Function, Mapping, SpecialForm, Value};

/****************\
|* Type aliases *|
\****************/
/// `Vars` are our AST nodes, represented as a pointer to an
/// expression
pub type Var = Rc<Expr>;

/// An S-expression is a slice of Vars
/// This will always show up in the form &SExpr
pub type SExpr = [Var];

/// Like SExpr, but owned
pub type OwnedSExpr = Vec<Var>;

/***************\
|* Expressions *|
\***************/
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

    pub fn new_var(self) -> Var {
        Rc::new(self)
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
        match self {
            Expr::Symbol(name) => Ok(name),
            _other => {
                Err(EvalError::Syntax {
                    expected: "Symbol".to_string(),
                    actual: self.type_str().to_string(),
                })
            },
        }
    }

    pub fn expect_keyword(&self) -> Result<&str, EvalError> {
        match self {
            Expr::Keyword(name) => Ok(name),
            _other => {
                Err(EvalError::Syntax {
                    expected: "Keyword".to_string(),
                    actual: self.type_str().to_string(),
                })
            },
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
