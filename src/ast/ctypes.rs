use std::fmt::{Display, Formatter};

use thiserror::Error;

use super::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    Str(String),
    Char(char),
    Int(isize),
    Bytes(Vec<u8>),
    Float(f64),
    Bool(bool), // are `true` / `false` symbols or lits? Right now a lit.
    Nil,
}

impl CType {
    /// For convenience - you usually want to wrap a "bare" literal
    /// with an Expr::Lit
    pub fn expr(self) -> Expr {
        Expr::Lit(self)
    }
}

impl Display for CType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CType::Str(x) => x.fmt(f),
            CType::Char(x) => x.fmt(f),
            CType::Int(x) => x.fmt(f),
            CType::Bytes(_) => write!(f, "not implemented"),
            CType::Float(x) => x.fmt(f),
            CType::Bool(x) => x.fmt(f),
            CType::Nil => write!(f, "Nil"),
        }
    }
}

// convenience: fallible conversion from Expr to CType
#[derive(Error, Debug)]
#[error("Expression '{expression:#?}' is not a literal.")]
pub struct NotALiteral {
    pub expression: Expr,
}

impl<'a> TryFrom<&'a Expr> for &'a CType {
    type Error = NotALiteral;

    fn try_from(expr: &'a Expr) -> Result<Self, Self::Error> {
        if let Expr::Lit(ctype) = expr {
            Ok(ctype)
        } else {
            Err(NotALiteral {
                expression: expr.clone(), // TODO: BAD! NO!
            })
        }
    }
}

/**********************************************\
|* Converting between rust values and CTypes  *|
\*********************************************/
#[derive(Error, Debug)]
#[error("Can't convert builtin type '{builtin_type}' to rust type '{rust_type}'")]
pub struct ConversionError {
    pub builtin_type: String,
    pub rust_type: String,
}

// These macros write out a series of TryFrom and From implementations that
// establish a mapping between certain rust values and our CTypes.
//
// The `From<rust_type> for CType` traits can be infallibly defined, because each
// rust type is associated with at most one CType variant.
//
// However, because enum variants aren't types in themselves, we can't define
// `From<Ctype> for rust_type - we can't guarantee that a given ctype can be
// converted into any specific rust type. Thus, we end up with
// `TryFrom<Ctype> for rust_type`, which at least makes it easy to _try_ to do the
// conversion and handle errors if not possible.
macro_rules! impl_ctype_conversions {
    ($($t:ty, $v:ident);* $(;)?) => {
        $(
            impl TryFrom<CType> for $t {
                type Error = ConversionError;

                fn try_from(ctype: CType) -> Result<Self, Self::Error> {
                    if let CType::$v(value) = ctype {
                        Ok(value)
                    } else {
                        Err(ConversionError{
                            builtin_type: "CType::$v".to_string(),
                            rust_type:"$t".to_string()
                        })
                    }
                }
            }

            impl From<$t> for CType {
                fn from(val: $t) -> Self {
                    CType::$v(val)
                }
            }
        )*
    };
}

// The mappings.
// The first element is the rust type, the second is the CType variant.
impl_ctype_conversions! {
    String, Str;
    char, Char;
    f64, Float;
    Vec<u8>, Bytes;
    isize, Int;
    bool, Bool;
}

/**********************************************\
|* Converting between rust values and &Expr   *|
\*********************************************/

// same as above, but lets us convert between expressions.
macro_rules! impl_expr_conversions {
    ($($t:ty, $v:ident);* $(;)?) => {
        $(
            impl<'a> TryFrom<&Expr> for $t {
                type Error = ConversionError;

                fn try_from(expr: &Expr) -> Result<Self, Self::Error> {
                    if let Expr::Lit(CType::$v(value)) = expr {
                        Ok(value.clone())
                    } else {
                        Err(ConversionError{
                            builtin_type: "Expr::Lit(CType::$v)".to_string(),
                            rust_type:"$t".to_string()
                        })
                    }
                }
            }

            impl From<$t> for Expr {
                fn from(val: $t) -> Self {
                    Expr::Lit(CType::from(val))
                }
            }
        )*
    };
}

// The mappings.
// The first element is the rust type, the second is $v in `Expr::Lit(CType::$v)`.
impl_expr_conversions! {
    String, Str;
    char, Char;
    f64, Float;
    Vec<u8>, Bytes;
    isize, Int;
    bool, Bool;
}
