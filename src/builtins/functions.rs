use std::iter::Map;
use std::rc::Rc;

use itertools::Itertools;

use crate::ast::Expr::Record;
use crate::ast::{
    Arity, CallForm, Expr, Function, InternalError, Mapping, OwnedSExpr, SExpr,
    Value, Var,
};
use crate::{EResult, EvalError, Scope};

pub(super) trait BuiltinFnBuilder {
    fn register(scope: &mut Scope) {
        let names = Self::names();
        let form: Var = Expr::Function(Function {
            name: names.first().unwrap().to_string(),
            arity: Self::arity(),
            arguments: Self::arguments()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            form: CallForm::Builtin(Self::eval),
        })
        .new_var();

        names
            .into_iter()
            .for_each(|s| scope.set(s, form.clone()))
    }

    /// names to bind to this function
    fn names() -> Vec<&'static str>;

    /// names of the function's arguments
    fn arguments() -> Vec<&'static str>;

    /// variadic or fixed function arity
    fn arity() -> Arity;

    /// Callback to evaluate a call to the function.
    /// Will be be passed an s-exp of its arguments' values.
    fn eval(sexpr: &SExpr) -> EResult<Var>;
}

/************\
|* Identity *|
\************/
pub(super) struct IdentityFnBuilder {}
impl BuiltinFnBuilder for IdentityFnBuilder {
    fn names() -> Vec<&'static str> {
        vec!["I", "echo"]
    }

    fn arguments() -> Vec<&'static str> {
        vec!["s"]
    }

    fn arity() -> Arity {
        Arity::Variadic
    }

    fn eval(args: &SExpr) -> EResult<Var> {
        Ok(Expr::SExpr(Vec::from(args)).new_var())
    }
}

/*********\
|* Print *|
\*********/
pub(super) struct PrintFnBuilder {}
impl BuiltinFnBuilder for PrintFnBuilder {
    fn names() -> Vec<&'static str> {
        vec!["print"]
    }

    fn arguments() -> Vec<&'static str> {
        vec!["s"]
    }

    fn arity() -> Arity {
        Arity::Fixed(1) // TODO: make it variadic
    }

    fn eval(args: &SExpr) -> EResult<Var> {
        let expression = args.first().unwrap().as_ref();
        let val: &Value = expression.try_into()?;
        let s: &str = val.try_into()?;
        println!("{s}");
        Ok(Rc::new(Expr::empty()))
    }
}

/*********\
|* First *|
\*********/
pub(super) struct FirstFnBuilder {}
impl BuiltinFnBuilder for FirstFnBuilder {
    fn names() -> Vec<&'static str> {
        vec!["first", "car"]
    }

    fn arguments() -> Vec<&'static str> {
        vec!["s-exp"]
    }

    fn arity() -> Arity {
        Arity::Fixed(1)
    }

    fn eval(args: &SExpr) -> EResult<Var> {
        if let Some(arg) = args.first() {
            Ok(arg.clone())
        } else {
            Ok(Expr::empty().new_var())
        }
    }
}

/*****************\
|* Concatenation *|
\*****************/
pub(super) struct ConcatFnBuilder {}
impl BuiltinFnBuilder for ConcatFnBuilder {
    fn names() -> Vec<&'static str> {
        vec!["concat"]
    }

    fn arguments() -> Vec<&'static str> {
        vec!["s-exp1", "s-exp2"]
    }

    fn arity() -> Arity {
        Arity::Fixed(2)
    }

    fn eval(args: &SExpr) -> EResult<Var> {
        let first = args.get(0).unwrap().expect_sexp()?;
        let second = args.get(1).unwrap().expect_sexp()?;

        Ok(Expr::SExpr(
            first
                .iter()
                .chain(second.iter())
                .map(Rc::clone)
                .collect::<OwnedSExpr>(),
        )
        .new_var())
    }
}

/*********\
|* Rest *|
\*********/
pub(super) struct RestFnBuilder {}
impl BuiltinFnBuilder for RestFnBuilder {
    fn names() -> Vec<&'static str> {
        vec!["rest", "cdr"]
    }

    fn arguments() -> Vec<&'static str> {
        vec!["s-exp"]
    }

    fn arity() -> Arity {
        Arity::Fixed(1)
    }

    fn eval(args: &SExpr) -> EResult<Var> {
        Ok(Expr::SExpr(
            args.first().unwrap().expect_sexp()?[1..]
                .iter()
                .map(Rc::clone)
                .collect(),
        )
        .new_var())
    }
}

/******************\
|* Record builder *|
\******************/
/// Builds a mapping (aka record)
/// TODO: Something not right here.
///     Shouldn't need to quote the arguments (also
///     this means symbols get passed unevaluated,
///     which is not right ...)
///     Q: Should this be a special form so we can control the evaluation?
///     Q: Or should I just make a special syntax for k/v pairs?
///
///  Currently you could write this as:
/// `(record (echo :key1 val1) (echo :key2 val2)) [...])`
/// Which sucks and is stupid to look at but otherwise works.
pub(super) struct RecordFnBuilder {}
impl BuiltinFnBuilder for RecordFnBuilder {
    fn names() -> Vec<&'static str> {
        vec!["record"]
    }

    fn arguments() -> Vec<&'static str> {
        vec!["kv_pairs"]
    }

    fn arity() -> Arity {
        Arity::Variadic
    }

    fn eval(args: &SExpr) -> EResult<Var> {
        args.iter()
            .map(|v| {
                v.expect_sexp_with_len(2)
                    .and_then(|vec| {
                        Ok((
                            vec.get(0)
                                .unwrap()
                                .expect_keyword()?
                                .to_owned(),
                            vec.get(1).unwrap().clone(),
                        ))
                    })
            })
            .collect::<EResult<Mapping>>()
            .map(|mapping| Record(mapping).new_var())
    }
}

/*********\
|* Add   *|
\*********/
pub(super) struct AddFnBuilder {}
impl BuiltinFnBuilder for AddFnBuilder {
    fn names() -> Vec<&'static str> {
        vec!["add", "+"]
    }

    fn arguments() -> Vec<&'static str> {
        vec!["x", "y"]
    }

    fn arity() -> Arity {
        Arity::Fixed(2)
    }

    /// temporary add implementation
    /// This needs a type system to do dispatch for us.
    fn eval(args: &SExpr) -> EResult<Var> {
        let ctypes: Vec<&Value> = args
            .iter()
            .map(|expr| expr.as_ref().try_into())
            .collect::<Result<Vec<&Value>, InternalError>>()?;

        assert_eq!(ctypes.len(), 2);
        let ct1 = ctypes.get(0).unwrap();
        let ct2 = ctypes.get(1).unwrap();

        // awful, just awful.
        let new_val = match (ct1, ct2) {
            // str | char, str | char
            (Value::Str(s1), Value::Str(s2)) => {
                Ok(Value::Str(s1.to_string() + s2))
            },
            (Value::Char(c1), Value::Char(c2)) => {
                Ok(Value::Str(c1.to_string() + &c2.to_string()))
            },
            (Value::Str(s1), Value::Char(c2)) => {
                Ok(Value::Str(s1.to_string() + &c2.to_string()))
            },
            (Value::Char(c1), Value::Str(s2)) => {
                Ok(Value::Str(c1.to_string() + s2))
            },

            // int | float, int | float
            (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 + i2)),
            (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 + f2)),
            (Value::Int(i1), Value::Float(f2)) => {
                Ok(Value::Float((*i1 as f64) + f2))
            },
            (Value::Float(f1), Value::Int(i2)) => {
                Ok(Value::Float(f1 + (*i2 as f64)))
            },

            // not supported
            _ => {
                Err(EvalError::Type {
                    actual: format!("{ct1:#?} + {ct2:#?}"),
                    expected: "Supported addition".to_string(),
                })
            },
        }?;

        Ok(Rc::new(Expr::Value(new_val)))
    }
}
