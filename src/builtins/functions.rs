use std::rc::Rc;

use crate::ast::{
    Arity, CallForm, Expr, Function, InternalError, SExpr, Value, Var,
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

    fn names() -> Vec<&'static str>;
    fn arguments() -> Vec<&'static str>;
    fn arity() -> Arity;
    fn eval(sexpr: &SExpr) -> EResult<Var>;
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
    /// This needs a type system to do dispath for us.
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
