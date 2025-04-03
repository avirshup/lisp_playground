use std::fmt::{Display, Formatter};

use super::Expr;
use crate::InternalError;

// TODO: values make sense as their own class of entities
//     *in the AST*. However at runtime they're annoying,
//     these types should be unioned with our expression enum.
//     But that would require a non-treewalk interpreter ...
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Char(char),
    Int(isize),
    Bytes(Vec<u8>),
    Float(f64),
    Bool(bool), // are `true` / `false` symbols or lits? Right now a lit.
    Nil,
}

impl Value {
    /// For convenience - you usually want to wrap a "bare" literal
    /// with an Expr::Lit
    pub fn expr(self) -> Expr {
        Expr::Value(self)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Str(x) => x.fmt(f),
            Value::Char(x) => x.fmt(f),
            Value::Int(x) => x.fmt(f),
            Value::Bytes(_) => write!(f, "not implemented"),
            Value::Float(x) => x.fmt(f),
            Value::Bool(x) => x.fmt(f),
            Value::Nil => write!(f, "Nil"),
        }
    }
}

/******************************\
|* Rust types <-> Value types *|
\******************************/
// These macros write out a series of TryFrom and From implementations that
// establish a mapping between certain rust values and our `Value`s.
//
// The `From<rust_type> for CType` traits can be infallibly defined, because each
// rust type is associated with at most one CType variant.
//
// However, because enum variants aren't types in themselves, we can't define
// `From<Ctype> for rust_type - we can't guarantee that a given ctype can be
// converted into any specific rust type. Thus, we end up with
// `TryFrom<Ctype> for rust_type`, which at least makes it easy to _try_ to do the
// conversion and handle errors if not possible.
macro_rules! impl_value_conversions {
    ($($t:ty, $v:ident);* $(;)?) => {
        $(
            impl TryFrom<&Value> for $t {
                type Error = InternalError;

                fn try_from(value: &Value) -> Result<Self, Self::Error> {
                    if let Value::$v(native_val) = &value {
                        Ok(native_val.clone())
                    } else {
                        Err(InternalError::Conversion{
                            builtin_type: "CType::$v".to_string(),
                            rust_type:"$t".to_string()
                        })
                    }
                }
            }

            impl From<$t> for Value {
                fn from(val: $t) -> Self {
                    Value::$v(val)
                }
            }
        )*
    };
}

// The mappings.
// The first element is the rust type, the second is the Value variant.
impl_value_conversions! {
    String, Str;
    char, Char;
    f64, Float;
    Vec<u8>, Bytes;
    isize, Int;
    bool, Bool;
}

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = InternalError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Value::Str(native_s) = value {
            Ok(native_s)
        } else {
            Err(InternalError::Conversion {
                builtin_type: "Value::Str".to_string(),
                rust_type: "&str".to_string(),
            })
        }
    }
}
