use std::iter::zip;
use std::rc::Rc;

use crate::ast::{Expr, SExpr};
use crate::errors::EvalError;
use crate::eval::{capture_variables, eval};
use crate::scope::Scope;
use crate::{ast, errors};

/// Set a symbol to a value in the current scope
fn set(args: &SExpr], scope: &mut Scope) -> errors::EResult<Var> {
    let symbol_name = args.get(0).unwrap().expect_symbol()?;
    let rhs = args.get(1).unwrap();

    let value = eval(rhs.clone(), scope)?;
    scope.set(symbol_name, value);

    Ok(Rc::new(Expr::empty()))
}

/// 2-ary function that defines a symbol in the current scope.
/// Behaves differently depending on the first argument:
/// 1) If the first argument is a Symbol, then it's equivalent to `set`)
/// 2) If the second argument is an S-expr of symbols, it defines a function
///  Specifically, the following two expressions are equivalent:
///     `(define (f a1 a2 ...) (b0 b1 b2 ...))`
///     `(set f (lambda a1 a2 ...) (b0 b1 b2 ...))`
fn define(args: &SExpr], scope: &mut Scope) -> errors::EResult<Var> {
    let lhs = args.get(0).unwrap();
    let rhs = args.get(1).unwrap();

    match lhs.as_ref() {
        Expr::Symbol(_name) => set(args, scope),
        Expr::SExpr(sexp) => {
            let lambda_args = vec![Rc::new(Expr::SExpr(sexp[1..].to_vec())), rhs];
            lambda(&lambda_args, scope).map(Expr::Proc)
        },
        _other => Err(EvalError::NotCallable(lhs.clone())),
    }
}

/// Create a callable lambda expression
fn lambda(args: &SExpr], scope: &mut Scope) -> errors::EResult<Var> {
    assert_eq!(args.len(), 2);
    let params = args.get(0).unwrap().expect_sexp()?;
    let body = args.get(1).unwrap().expect_sexp()?;

    // get arguments, make sure they're all actually symbols
    let param_names: Vec<&str> = params
        .iter()
        .map(|expr| {
            match expr.as_ref() {
                Expr::Symbol(name) => Ok(name),
                _other => {
                    Err(EvalError::Type {
                        expected: "Symbol".to_string(),
                        actual: expr.type_str().to_string(),
                    })
                },
            }
        })
        .collect()?;

    // TODO: scope lifetimes ... sigh
    // Create a scope that maps the function arguments symbols back to themselves
    let mut arg_scope = Scope::child(Rc::new(scope));
    zip(param_names.iter(), params.iter()).for_each(|(name, param)| {
        arg_scope.set(name, param.clone());
    });

    // Capture all symbols into the s-expression, build a closure that
    // evaluates it
    //
    // TODO: lambdas are pretty important, they deserved a named function
    //  to execute them. This is nasty here `Proc`s, as defined, are really
    //  about calling core _rust_ functions, rather than evaluating
    //  s-expressions. I think it makes sense to treat these as a first-class
    //  object in themselves ... Maybe call these `Expr::Lambda`?
    let captured_sexpr = capture_variables(body, &arg_scope)?;
    let eval_fn = move |args: &SExpr]| {
        let mut scope = Scope::empty();
        zip(param_names.iter(), params.iter()).for_each(|(name, param)| {
            scope.set(name, param.clone());
        });
        // TODO: should not need to make scope mutable here
        eval(captured_sexpr, &mut scope)
    };

    Ok(Rc::new(Expr::Proc(ast::Proc {
        name: None,
        arity: ast::Arity::Fixed(params.len()),
        eval: eval_fn,
    })))
}
