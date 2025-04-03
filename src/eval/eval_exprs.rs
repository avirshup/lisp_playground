use crate::ast::errors::{EResult, EvalError};
use crate::ast::{Arity, CallForm, Expr, Function, OwnedSExpr, SExpr, Var};
use crate::scope::Scope;

/// Evaluate an expression. Handles a few cases:
/// 1) If it's an s-expression, it's evaluated (see eval_sexpr, below);
/// 2) if it's a symbol, it's retrieved from the current scope;
/// 3) all other expression types are returned unchanged.
///
/// Note that `eval_sexpr` usually needs to evaluate its arguments,
/// which means it will need to recursively call this function.
pub fn eval(var: &Var, scope: &mut Scope) -> EResult<Var> {
    match var.as_ref() {
        Expr::SExpr(sexpr) => eval_sexpr(sexpr, scope),
        Expr::Symbol(name) => scope.lookup_or_error(name),
        _ => Ok(var.clone()), // clones the Rc, not the value
    }
}

/// Evaluate an s-expression.
/// Handles 4 cases based on evaluating the first element of the list:
/// 1) an empty s-expression is returned unchanged;
/// 2) a special form is called with all arguments as-is (unevaluated), and
///    provided with a mutable reference to the scope;
/// 3) a proc is evaluated by calling `eval_proc`, below;
/// 4) everything else is a runtime error
pub fn eval_sexpr(sexpr: &SExpr, scope: &mut Scope) -> EResult<Var> {
    if sexpr.is_empty() {
        return Ok(Var::new(Expr::empty()));
    }

    // evaluate head
    let head = eval(sexpr.first().unwrap(), scope)?;

    // evaluate entire s-expression
    let tail = &sexpr[1..];
    match head.as_ref() {
        Expr::Special(special) => (special.eval)(tail, scope),
        Expr::Function(func) => {
            let eval_args: OwnedSExpr =
                tail.iter()
                    .map(|e| eval(e, scope))
                    .collect::<Result<OwnedSExpr, EvalError>>()?;

            eval_function(func, eval_args)
        },
        _ => {
            Err(EvalError::NotCallable(
                head.type_str().to_string(),
            ))
        },
    }
}

/// Evaluate a function call by first evaluating all arguments, then
/// sending the array of evaluated arguments to the proc.
///
/// Note that, unlike special forms, functions don't have any access to the scope.
/// Of course scope will be accessed while evaluating the arguments,
/// including special forms trhat may potentially modify it.
pub fn eval_function(func: &Function, eval_args: OwnedSExpr) -> EResult<Var> {
    check_arity(&func.arity, &func.name, eval_args.len())?;

    match &func.form {
        CallForm::Builtin(f) => f(&eval_args),
        CallForm::Lambda { sexpr, scope } => {
            let mut arg_scope = scope.bind_args(&func.arguments, &eval_args);
            eval_sexpr(sexpr, &mut arg_scope)
        },
    }
}

fn check_arity(
    arity: &Arity,
    name: &str,
    n_args: usize,
) -> Result<(), EvalError> {
    if let &Arity::Fixed(arity) = arity {
        if n_args != arity {
            return Err(EvalError::Arity {
                name: name.to_string(),
                arity,
                num_args_provided: n_args,
            });
        }
    };

    Ok(())
}
