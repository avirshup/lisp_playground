use crate::ast::{Expr, SExpr, SpecialForm, Var};
use crate::{EResult, Scope};

/// Lexical symbol binding for closures
/// - i.e., captures variables from the enclosing scope.
///
/// This will be kicked off by anything that defines a closure
/// (`lambda` / `define`, in particular).
///
/// See **_extensive_** discussion in LEXICAL_BINDING.md.
pub fn capture_sexp_references(
    sexpr: &SExpr,
    outer_scope: &Scope,
    capture_scope: &mut Scope,
) -> EResult<()> {
    if let Some((special_var, maybe_name)) = is_special(sexpr, outer_scope) {
        let special: &SpecialForm = special_var.expect_special()?; // TODO: this should be an _internal_ error

        if let Some(name) = maybe_name {
            capture_scope.set(&name, special_var.clone())
        }

        // if s-expr is a special form, delegate to its bind_outer_scope method
        (special.bind_outer_scope)(&sexpr[1..], outer_scope, capture_scope)
    } else {
        // capture references for each s-xep
        for var in sexpr.iter() {
            match var.as_ref() {
                Expr::SExpr(inner_sexpr) => {
                    capture_sexp_references(
                        inner_sexpr,
                        outer_scope,
                        capture_scope,
                    )?
                },
                Expr::Symbol(_) => {
                    capture_symbol_reference(
                        var.clone(),
                        outer_scope,
                        capture_scope,
                    )?
                },
                _ => (),
            }
        }
        Ok(())
    }
}

/// Capture a not-yet defined symbol from the outer scope,
/// unless it will be provided as an argument.
///
/// TODO: Right now, we signal that a symbol will be provided within the scope
///     by making it tautological - i.e., equal to itself. This is cute and all
///     but probably this should be signaled explicitly with a real sentinel;
///     I'm sure this method breaks with some meta-programming edge case or
///     something.
fn capture_symbol_reference(
    symbol: Var,
    outer_scope: &Scope,
    capture_scope: &mut Scope,
) -> EResult<()> {
    let name = symbol.expect_symbol()?;
    if !capture_scope.has(name) {
        let outer_val = outer_scope.lookup_or_error(name)?;

        // don't capture it if it's tautological
        if outer_val == symbol {
            return Ok(());
        }

        // otherwise add it to our collection of captured variables
        capture_scope.set(name, outer_val)
    }
    Ok(())
}

/// check if an s-expression is a call to a special form.
/// If so, return the form, and, if necessary, the symbol to bind the form to.
///
/// Note that this takes advantage of the syntax rule that `Expr::Special` may
/// not be aliased or shadowed, or returned from functions;
/// thus, even at this lexical analysis stage, we know what is
/// and isn't a special form.
fn is_special(sexpr: &SExpr, scope: &Scope) -> Option<(Var, Option<String>)> {
    // Get first expression in the sexpr
    let Some(var) = sexpr.first() else {
        // it's empty
        return None;
    };
    let expr = var.as_ref();

    if let Expr::Special(..) = expr {
        // this is an "anonymous" (for our purposes) special form, stop here
        return Some((var.clone(), None));
    };
    let Expr::Symbol(s) = expr else {
        // it's not a Special or a Symbol, so this is not a special form
        return None;
    };
    if let Some(outer_val) = scope.lookup(s) {
        //
        if let Expr::Special(..) = outer_val.as_ref() {
            return Some((var.clone(), Some(s.clone())));
        }
    }
    None
}
