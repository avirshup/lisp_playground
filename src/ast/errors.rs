use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {}

/**********************************************\
|* Converting between rust values and CTypes  *|
\*********************************************/
#[derive(Error, Debug)]
pub enum InternalError {
    #[error(
        "Can't convert builtin type '{builtin_type}' to rust type '{rust_type}'"
    )]
    Conversion {
        builtin_type: String,
        rust_type: String,
    },

    #[error("Expression '{expression}' is not a literal.")]
    NotAValue { expression: String },
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error(transparent)]
    Internal(#[from] InternalError),

    #[error("syntax error: expected {expected}, got {actual}")]
    Syntax { expected: String, actual: String },

    #[error("type error: expected {expected}, got {actual}")]
    Type { expected: String, actual: String },

    #[error("Could not find symbol '{0}'")]
    LookupError(String),

    #[error("First item in S-expr is not a proc or special: '{0:#?}'")]
    NotCallable(String),

    #[error(
        "Function {name} takes {arity} arguments but got {num_args_provided}"
    )]
    Arity {
        name: String,
        arity: usize,
        num_args_provided: usize,
    },
}

pub type EResult<T> = Result<T, EvalError>;
