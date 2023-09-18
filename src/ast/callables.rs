use std::fmt::{Display, Formatter};
use std::iter::repeat;

use super::{EResult, OwnedSExpr, SExpr, Var};
use crate::Scope;

/*****************\
|* Special forms *|
\*****************/
#[derive(Debug, Clone, PartialEq)]
pub struct SpecialForm {
    pub name: String,
    pub arity: Arity,

    /// Evaluate the special form
    pub eval: fn(&SExpr, &mut Scope) -> EResult<Var>,

    /// given the s-expressions arguments, return a list of
    /// variables that it needs from its enclosing scope.
    /// This method is, in particular, a hook for `lambda`
    /// (or anything building a closure) to call on any interior special forms.
    /// Q: is this enough? Do we need a real scope object?
    pub bind_outer_scope: fn(&SExpr, &Scope, &mut Scope) -> EResult<()>,
}

impl Display for SpecialForm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#special[{}]", self.name)
    }
}

/*************\
|* Functions *|
\*************/
// static mut COUNTER: usize = 0;

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: Arity,
    pub arguments: Vec<String>,
    pub form: CallForm,
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}[{}]",
            self.form.type_str(),
            self.name,
            self.arguments.join(",")
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallForm {
    Lambda { sexpr: OwnedSExpr, scope: Scope },
    Builtin(fn(&SExpr) -> EResult<Var>),
    // Curry({inner: Rc<CallForm>,
    //       bound: Scope}),
}

impl CallForm {
    fn type_str(&self) -> &'static str {
        match self {
            CallForm::Lambda { .. } => "λ",
            CallForm::Builtin(..) => "builtin",
            // CallForm::Curry(_) => "curry",
        }
    }
}

/*********************\
|* Common components *|
\*********************/
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Arity {
    Fixed(usize),
    Variadic,
}

impl Display for Arity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Arity::Fixed(n) => {
                write!(
                    f,
                    "({})",
                    repeat("_")
                        .take(*n)
                        .collect::<Vec<&str>>()
                        .join(",")
                )
            },
            Arity::Variadic => {
                write!(f, "(...)")
            },
        }
    }
}
