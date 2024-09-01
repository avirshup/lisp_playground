use std::rc::Rc;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::ast::{
    Arity, CallForm, Expr, Function, SExpr, SpecialForm, Value, Var,
};
use crate::builtins::functions::BuiltinFnBuilder;
use crate::{eval, EResult, EvalError, Scope};

/** See also:
  - https://clojure.org/reference/special_forms#var
  - https://docs.racket-lang.org/reference/syntax.html
  - http://www.lispworks.com/documentation/HyperSpec/Body/03_ababa.htm
**/

/// Helper trait for defining built-in special forms.
/// Note: currently we don't instantiate structs for any of these,
/// these traits are just namespaces to group the methods for each form.
pub(super) trait BuiltinSpecialBuilder {
    fn register(scope: &mut Scope) {
        let names = Self::names();
        let form: Var = Expr::Special(SpecialForm {
            name: names.first().unwrap().to_string(),
            arity: Self::arity(),
            eval: Self::eval,
            bind_outer_scope: Self::bind_outer_scope,
        })
        .new_var();

        names
            .into_iter()
            .for_each(|s| scope.set(s, form.clone()))
    }

    /// the built-in names that refer to this special form
    fn names() -> Vec<&'static str>;

    /// variadic or fixed arity
    fn arity() -> Arity;

    /// called with list of arguments and enclosing scope
    fn eval(sexpr: &SExpr, scope: &mut Scope) -> EResult<Var>;

    /// Builds the scope in which to evaluated this forms' arguments,
    /// if applicable. This is *early* binding - given the outer scope,
    /// it should update the "capture scope" with any values it needs to capture.
    ///
    /// Need not be implemented if not applicable to the special form;
    /// by default is a no-op.
    ///
    ///  # Arguments
    ///  - `args` - input: the form's _un-evaluated_ arguments.
    ///  - `scope` - input: the enclosing scope.
    ///  - `capture_scope` - in/out: the captured scope for evaluating the form's
    ///    arguments
    fn bind_outer_scope(
        _args: &SExpr,
        _scope: &Scope,
        _capture_scope: &mut Scope,
    ) -> EResult<()> {
        Ok(())
    }
}

/******************************\
|* "If" special form impl     *|
\******************************/
pub(super) struct IfFormBuilder;
impl BuiltinSpecialBuilder for IfFormBuilder {
    fn names() -> Vec<&'static str> {
        vec!["if"]
    }

    fn arity() -> Arity {
        Arity::Fixed(3)
    }

    /// Evaluate 1st argument then _either_ the 2nd or 3rd argument, not both
    fn eval(args: &SExpr, scope: &mut Scope) -> EResult<Var> {
        let determinant = eval(args.first().unwrap(), scope)?;
        let Expr::Value(Value::Bool(result)) = determinant.as_ref() else {
            return Err(EvalError::Type {
                expected: "Bool".to_string(),
                actual: determinant.type_str().to_string(),
            });
        };

        let idx: usize = if *result { 1 } else { 2 };
        eval(args.get(idx).unwrap(), scope)
    }

    /// capture references for all arguments
    fn bind_outer_scope(
        args: &SExpr,
        scope: &Scope,
        capture_scope: &mut Scope,
    ) -> EResult<()> {
        eval::bind_sexpr_outer_scope(args, scope, capture_scope)
    }
}

/******************************\
|* "Quote" special form impl *|
\******************************/
pub(super) struct QuoteFormBuilder;
impl BuiltinSpecialBuilder for QuoteFormBuilder {
    fn names() -> Vec<&'static str> {
        vec!["quote"]
    }

    fn arity() -> Arity {
        Arity::Variadic
    }

    fn eval(args: &SExpr, _scope: &mut Scope) -> EResult<Var> {
        Ok(Expr::SExpr(Vec::from(args)).new_var())
    }

    // TODO: binds nothing, right? Not 100% sure
    // fn bind_outer_scope(
    //     _args: &SExpr,
    //     _scope: &Scope,
    //     _capture_scope: &mut Scope,
    // ) -> EResult<()> {panic!("at the disco")}
}

/******************************\
|* "defvar" special form impl *|
\******************************/
pub(super) struct DefVarForm;
impl BuiltinSpecialBuilder for DefVarForm {
    fn names() -> Vec<&'static str> {
        vec!["defvar"]
    }

    fn arity() -> Arity {
        Arity::Fixed(2)
    }

    fn eval(args: &SExpr, scope: &mut Scope) -> EResult<Var> {
        let symbol_name = args.get(0).unwrap().expect_symbol()?;
        let body = args.get(1).unwrap();

        let value = eval(body, scope)?;
        scope.set(symbol_name, value);

        Ok(Rc::new(Expr::empty()))
    }

    fn bind_outer_scope(
        args: &SExpr,
        scope: &Scope,
        capture_scope: &mut Scope,
    ) -> EResult<()> {
        let symbol = args.get(0).unwrap();
        let symbol_name = symbol.expect_symbol()?;
        let rhs = args.get(1).unwrap();

        // capture any variables necessary to evaluate the RHS
        eval::bind_outer_scope(rhs, scope, capture_scope)?;

        // if not already part of the closure, make this
        // symbol its own _Symbol_ in the capture scope (?)
        // TODO: this is probably incorrect if you are, for instance, running
        //      (define) within the scope of a thing.
        if !capture_scope.has(symbol_name) {
            capture_scope.set(symbol_name, symbol.clone());
        }
        Ok(())
    }
}

/******************************\
|* "Define" special form impl *|
\******************************/
/// Binds symbols to expressions in the current scope.
/// - If the first argument is a symbol, evaluates the second argument then binds
///   it to the symbol.
/// 2) If the second argument is an S-expr of symbols, it defines a function
///  Specifically, the following two expressions are equivalent:
///     `(define (f a1 a2 ...) (b0 b1 b2 ...))`
///     `(define f (lambda a1 a2 ...) (b0 b1 b2 ...))`
pub(super) struct DefineFormBuilder;
impl BuiltinSpecialBuilder for DefineFormBuilder {
    fn names() -> Vec<&'static str> {
        vec!["def", "define"]
    }

    fn arity() -> Arity {
        Arity::Fixed(2)
    }

    fn eval(args: &SExpr, scope: &mut Scope) -> EResult<Var> {
        let lhs = args.get(0).unwrap();
        let rhs = args.get(1).unwrap();

        match lhs.as_ref() {
            // treat as equivalent to DefVar
            Expr::Symbol(_name) => DefVarForm::eval(args, scope),

            // treat as equivalent to (defvar #0 (lambda ...))
            Expr::SExpr(sexp) => {
                // get the function name
                let fn_name = sexp
                    .first()
                    .ok_or(EvalError::Syntax {
                        expected: "List of symbols".to_string(),
                        actual: "Empty".to_string(),
                    })?
                    .expect_symbol()?;

                // arguments for `lambda`
                let lambda_args = vec![
                    Expr::SExpr(
                        sexp[1..]
                            .iter()
                            .map(Rc::clone)
                            .collect(),
                    )
                    .new_var(),
                    rhs.clone(),
                ];

                // move new function into scope
                let form = LambdaFormBuilder::build_function(
                    fn_name.to_string(),
                    &lambda_args,
                    scope,
                )?;

                scope.set(fn_name, form);

                Ok(Expr::empty().new_var())
            },
            _other => {
                Err(EvalError::Syntax {
                    expected: "S-Expression or Symbol".to_string(),
                    actual: lhs.type_str().to_string(),
                })
            },
        }
    }

    fn bind_outer_scope(
        args: &SExpr,
        scope: &Scope,
        capture_scope: &mut Scope,
    ) -> EResult<()> {
        // TODO: this is almost an exact duplicate of eval, except it has
        // different args
        let lhs = args.get(0).unwrap();
        let rhs = args.get(1).unwrap();

        match lhs.as_ref() {
            Expr::Symbol(_name) => {
                DefVarForm::bind_outer_scope(args, scope, capture_scope)
            },
            Expr::SExpr(sexp) => {
                let lambda_args =
                    vec![Rc::new(Expr::SExpr(sexp[1..].to_vec())), rhs.clone()];
                LambdaFormBuilder::bind_outer_scope(
                    &lambda_args,
                    scope,
                    capture_scope,
                )
            },
            _other => {
                Err(EvalError::Syntax {
                    expected: "S-Expression or Symbol".to_string(),
                    actual: lhs.type_str().to_string(),
                })
            },
        }
    }
}

/******************************\
|* "Lambda" special form impl *|
\******************************/
/// Given the argument list in a function/lambda declaration.
/// 1) check that it is in fact a list of symbol names, and then
/// 2) return them as a vector
/// TODO: this might need to go somewhere more public?
pub(super) struct LambdaFormBuilder;

impl LambdaFormBuilder {
    fn get_argnames(expr: &Expr) -> Result<Vec<String>, EvalError> {
        expr.expect_sexp().and_then(|sexpr| {
            sexpr
                .iter()
                .map(|expr| expr.expect_symbol().map(String::from))
                .collect()
        })
    }

    fn build_function(
        name: String,
        sexpr: &SExpr,
        scope: &mut Scope,
    ) -> EResult<Var> {
        // capture references to outer scope
        let mut capture_scope = Scope::new(None);
        LambdaFormBuilder::bind_outer_scope(sexpr, scope, &mut capture_scope)?;

        // create function object
        let argnames = Self::get_argnames(sexpr.get(0).unwrap())?;
        let body = sexpr.get(1).unwrap().expect_sexp()?;
        Ok(Rc::new(
            Function {
                name,
                arity: Arity::Fixed(argnames.len()),
                arguments: argnames,
                form: CallForm::Lambda {
                    sexpr: Vec::from(body),
                    scope: capture_scope,
                },
            }
            .into(),
        ))
    }
}

// a counter for generating unique names for our lambdas
lazy_static! {
    static ref LAMBDA_COUNTER: Mutex<usize> = Mutex::new(0);
}

impl BuiltinSpecialBuilder for LambdaFormBuilder {
    fn names() -> Vec<&'static str> {
        vec!["lambda", "λ"]
    }

    fn arity() -> Arity {
        Arity::Fixed(2)
    }

    fn eval(sexpr: &SExpr, scope: &mut Scope) -> EResult<Var> {
        // TODO: deal with LAMBDA_COUNTER overflow
        // (should just restart at 0)
        let name = {
            let mut count = LAMBDA_COUNTER.lock().unwrap();
            *count += 1;
            format!("λ_{count}")
        };

        Self::build_function(name, sexpr, scope)
    }

    /// find names of outer vars that this thing requires.
    /// Note that the scope is only used to identify special forms at this point!
    /// Although we _should_ go ahead and create the bindings directly.
    /// Confused: do we need to send 2 scopes? And have it transfer _from_
    /// one into the other?
    /// Or do we need to
    fn bind_outer_scope(
        sexpr: &SExpr,
        outer_scope: &Scope,
        capture_scope: &mut Scope,
    ) -> Result<(), EvalError> {
        // get arguments and function body
        let argnames = Self::get_argnames(sexpr.get(0).unwrap())?;
        let body = sexpr.get(1).unwrap().expect_sexp()?;

        let mut child_outer = outer_scope.child();
        for name in argnames.into_iter() {
            child_outer.set(&name, Expr::Symbol(name.clone()).new_var())
        }

        eval::bind_sexpr_outer_scope(body, &child_outer, capture_scope)
    }
}

/******************************\
|* BELOW: unimplemented ideas *|
\******************************/

// IDEA: Keyword args with swift-style caller/callee names.
//     They keyword arg is what the caller sees, but the variable name
//     is what the function sees?
//     See also Clojure version: https://clojure.org/news/2021/03/18/apis-serving-people-and-programs
//     Dumb Q: should that go here or in the AST/parser?
//     A: obviously here, we'll need to attach enough information
//       to the function to remap the keyword args.

/******************************\
|* "special" builder form impl *|
\******************************/

// `special` is a built-in special form creator, with nearly identical
// syntax and semantics to the `lambda` form.
// The differences are simply what happens later, when the newly defined
// form or function is _called_:
//  - unlike functions, arguments to special forms are _not_ evaluated when being
//    sent to special forms
// Q: should special forms receive the scope as a first class object?
//    Do users need to provide a "close_over" method somehow?
//    Are user-defined specials even a good idea? What can they do
//    that `lambda`s operating on quoted arguments can't do?
//
// struct SpecialBuilderForm;
// impl BuiltinSpecial for SpecialBuilderForm {
//     fn names() -> Vec<&'static str> {
//         vec!["special"]
//     }
//
//     fn arity() -> Arity {
//         Arity::Fixed(2)
//     }
//
//     fn eval(sexpr: &SExpr, scope: &mut Scope) -> EResult {
//         let argnames = get_argnames(sexpr.get(0).unwrap())?;
//         let body = sexpr.get(1).unwrap().expect_sexp()?;
//         let scope = LambdaForm::close_over(&body, &scope)?;
//
//         Ok(Rc::new(
//             SpecialForm {
//                 name: "special".to_string(),
//                 arity: Arity::Fixed(argnames.len()),
//                 arguments: argnames,
//                 form: CallForm::Lambda {
//                     sexpr: Vec::from(body),
//                     scope,
//                 },
//             }
//             .into(),
//         ))
//     }
//
//     fn close_over(
//         sexpr: &SExpr,
//         outer_scope: &Scope,
//     ) -> Result<Scope, EvalError> { todo!()
//     }
// }

/*********************************\
|* "Programme" special form impl *|
\*********************************/
// A programme is a series of things to do in order
// struct ProgrammeForm;
// impl BuiltinSpecial for ProgrammeForm {
//     fn names() -> Vec<&'static str> {
//         vec!["programme", "program"]
//     }
//
//     fn arity() -> Arity {
//         Arity::Variadic
//     }
//
//     fn eval(sexpr: &SExpr, scope: &mut Scope) -> EResult {
//         todo!()
//     }
//
//     fn close_over(sexpr: &SExpr, scope: &Scope) -> Result<Scope, EvalError> {
//         todo!()
//     }
// }
