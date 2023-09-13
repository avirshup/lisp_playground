use std::rc::Rc;

use ctypes::NotALiteral;

use super::expressions::Expr;
use super::procs::{Arity, Proc, ProcResult};
use super::scope::Scope;
use crate::ctypes;
use crate::ctypes::{CType, ConversionError};
use crate::procs::ProcError;
use crate::procs::ProcError::TypeError;

/// Convenience - map a constant conversion failure into a type error
impl From<ConversionError> for ProcError {
    fn from(err: ConversionError) -> Self {
        TypeError {
            actual: err.builtin_type,
            // TODO: this should be mapped to the right CType name
            expected: err.rust_type,
        }
    }
}

/// Convenience - identify
impl From<NotALiteral> for ProcError {
    fn from(err: NotALiteral) -> Self {
        let expr = err.expression;
        TypeError {
            actual: format!("{expr:#?}"),
            expected: "Expr::Lit".to_string(),
        }
    }
}

pub fn builtins() -> Scope {
    let mut scope = Scope::empty();

    scope.set(
        "print",
        Rc::new(Expr::Proc(Proc {
            name: "print".to_string(),
            arity: Arity::Fixed(1),
            eval: print,
        })),
    );

    scope.set(
        "+",
        Rc::new(Expr::Proc(Proc {
            name: "+".to_string(),
            arity: Arity::Fixed(2),
            eval: add,
        })),
    );

    scope
}

/************\
|* Printing *|
\************/

/* Note that these don't need to check the number of args passed, arity
is automatically checked */

/// print a string to console
fn print(args: &[Rc<Expr>]) -> ProcResult {
    let expression = args.first().unwrap().as_ref();
    let s: String = expression.try_into()?;
    println!("{s}");
    Ok(Rc::new(Expr::empty()))
}

// TODO: implement conversion of all CTypes to strings

/********************\
|* Addition is hard *|
\********************/

/// Implements add for:
/// (Str | Char, Str | Char) -> Str
/// (Int, Int) -> Int
/// (Float | Int, Float | Int) -> Float (unless both are Ints)
/// (No notion of overloading / adding implementations, here anyway)
fn add(args: &[Rc<Expr>]) -> ProcResult {
    let ctypes: Vec<&CType> = args
        .into_iter()
        .map(|expr| expr.as_ref().try_into())
        .collect::<Result<Vec<&CType>, NotALiteral>>()?;

    assert_eq!(ctypes.len(), 2);
    add_ctypes(
        ctypes.get(0).unwrap(),
        ctypes.get(1).unwrap(),
    )
}

/// awful, just awful
fn add_ctypes(ct1: &CType, ct2: &CType) -> ProcResult {
    let new_val = match (ct1, ct2) {
        // str | char, str | char
        (CType::Str(s1), CType::Str(s2)) => Ok(CType::Str(s1.to_string() + s2)),
        (CType::Char(c1), CType::Char(c2)) => {
            Ok(CType::Str(c1.to_string() + &c2.to_string()))
        },
        (CType::Str(s1), CType::Char(c2)) => {
            Ok(CType::Str(s1.to_string() + &c2.to_string()))
        },
        (CType::Char(c1), CType::Str(s2)) => Ok(CType::Str(c1.to_string() + s2)),

        // int | float, int | float
        (CType::Int(i1), CType::Int(i2)) => Ok(CType::Int(i1 + i2)),
        (CType::Float(f1), CType::Float(f2)) => Ok(CType::Float(f1 + f2)),
        (CType::Int(i1), CType::Float(f2)) => Ok(CType::Float((*i1 as f64) + f2)),
        (CType::Float(f1), CType::Int(i2)) => Ok(CType::Float(f1 + (*i2 as f64))),

        // not supported
        _ => {
            Err(TypeError {
                actual: format!("{ct1:#?} + {ct2:#?}").to_string(),
                expected: "Supported addition".to_string(),
            })
        },
    }?;

    Ok(Rc::new(Expr::Lit(new_val)))
}
