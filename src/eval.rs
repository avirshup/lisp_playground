use std::rc::Rc;

use thiserror::Error;

use super::expressions::{Expr, SExpr};
use super::procs;
use super::scope::Scope;
use crate::eval::EvalError::NotAProc;
use crate::procs::Proc;

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("internal error: {0}")]
    InternalError(#[from] anyhow::Error),

    #[error("Could not find symbol '{0}'")]
    LookupError(String),

    #[error("First item in S-expr is not a proc or special: '{0:#?}'")]
    NotAProc(Rc<Expr>),

    #[error("Could not evaluate proc: {0}")]
    ProcError(#[from] procs::ProcError),
}

type EvalResult = Result<Rc<Expr>, EvalError>;

/// Evaluate an expression. Handles a few cases:
/// 1) If it's an s-expression, it's evaluated (see eval_sexpr, below);
/// 2) if it's a symbol, it's retrieved from the current scope;
/// 3) all other expression types are returned unchanged.
///
/// Note that `eval_sexpr` usually needs to evaluate its arguments,
/// which means it will need to recursively call this function.
pub fn eval(expr: Rc<Expr>, scope: &mut Scope) -> EvalResult {
    match expr.as_ref() {
        Expr::SExpr(sexpr) => eval_sexpr(&sexpr, scope),
        Expr::Symbol(name) => {
            scope
                .lookup(&name)
                .ok_or_else(|| EvalError::LookupError(name.clone()))
        },
        _ => Ok(expr.clone()), // clones the Rc, not the value
    }
}

/// Evaluate an s-expression.
/// Handles 4 cases based on evaluating the first element of the list:
/// 1) an empty s-expression is returned unchanged;
/// 2) a special form is called with all arguments as-is (unevaluated), and
///    provided with a mutable reference to the scope;
/// 3) a proc is evaluated by calling `eval_proc`, below;
/// 4) everything else is a runtime error
fn eval_sexpr(sexpr: &SExpr, scope: &mut Scope) -> EvalResult {
    if sexpr.is_empty() {
        return Ok(Rc::new(Expr::SExpr(Vec::new())));
    }

    // evaluate head
    let head_expr = sexpr.first().unwrap();
    let head = eval(head_expr.clone(), scope)?;

    // evaluate entire s-expression
    let tail = &sexpr[1..];
    match head.as_ref() {
        Expr::Special(special) => Ok(special.evaluate(tail, scope)?),
        Expr::Proc(proc) => Ok(eval_proc(proc, tail, scope)?),
        _ => Err(NotAProc(head.clone())),
    }
}

/// Evaluate a function call by first evaluating all arguments, then
/// sending the array of evaluated arguments to the proc.
///
/// Note that, unlike special forms, procs don't have any access to the scope.
/// Of course scope will be accessed while evaluating the arguments,
/// including special forms that may potentiall modify it.
fn eval_proc(proc: &Proc, args: &[Rc<Expr>], scope: &mut Scope) -> EvalResult {
    let eval_args: Vec<Rc<Expr>> = args
        .iter()
        .map(|e| eval(e.clone(), scope))
        .collect::<Result<Vec<Rc<Expr>>, EvalError>>()?;

    Ok(proc.evaluate(&eval_args)?)
}
