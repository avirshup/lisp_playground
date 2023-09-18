use super::{Expr, Function, InternalError, OwnedSExpr, SpecialForm, Value};

/*************************\
|* Exprs into Value types *|
\*************************/
impl<'a> TryFrom<&'a Expr> for &'a Value {
    type Error = InternalError;

    fn try_from(var: &'a Expr) -> Result<Self, InternalError> {
        if let Expr::Value(v) = var {
            Ok(v)
        } else {
            Err(InternalError::NotAValue {
                expression: format!("{}", var),
            })
        }
    }
}

/****************************\
|* Exprs from wrapped types *|
\****************************/
// Coercion sugar to make it easier to create exprs
macro_rules! impl_expr_from_type {
    ($($t:ty, $v:ident);* $(;)?) => {
        $(
            impl From<$t> for Expr {
                fn from(value: $t) -> Self {
                    Expr::$v(value)
                }
            }
        )*
    };
}

impl_expr_from_type!(
    Value, Value;
    OwnedSExpr, SExpr;
    Function, Function;
    SpecialForm, Special;
);

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
macro_rules! impl_ctype_conversions {
    ($($t:ty, $v:ident);* $(;)?) => {
        $(
            impl TryFrom<Value> for $t {
                type Error = InternalError;

                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    if let Value::$v(native_val) = value {
                        Ok(native_val)
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
// The first element is the rust type, the second is the CType variant.
impl_ctype_conversions! {
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
                rust_type: "&stsr".to_string(),
            })
        }
    }
}
