use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::zip;
use std::rc::Rc;

use crate::ast::{Expr, SExpr};
use crate::{EResult, EvalError};

#[derive(Debug, Clone, PartialEq)]
pub struct Scope(Rc<InnerScope>);

/// The actual scope data
#[derive(Debug, PartialEq)]
struct InnerScope {
    parent: Option<Scope>,
    symbols: RefCell<HashMap<String, Rc<Expr>>>,
}

impl Scope {
    pub fn new(parent: Option<Scope>) -> Self {
        Scope(Rc::new(InnerScope {
            parent,
            symbols: RefCell::new(HashMap::new()),
        }))
    }

    pub fn child(&self) -> Self {
        let parent = Scope(self.0.clone());

        Scope::new(Some(parent))
    }

    pub fn set(&mut self, key: &str, val: Rc<Expr>) {
        self.0
            .symbols
            .borrow_mut()
            .insert(key.to_string(), val);
    }

    pub fn has(&self, symbol: &str) -> bool {
        self.0
            .symbols
            .borrow()
            .contains_key(symbol)
    }

    pub fn lookup(&self, symbol: &str) -> Option<Rc<Expr>> {
        self.0
            .symbols
            .borrow()
            .get(symbol)
            .cloned()
            .or_else(|| {
                self.0
                    .parent
                    .as_ref()
                    .and_then(|parent| parent.lookup(symbol))
            })
    }

    /***********\
    |* Helpers *|
    \***********/
    pub fn lookup_or_error(&self, symbol: &str) -> EResult<Rc<Expr>> {
        self.lookup(symbol)
            .ok_or_else(|| EvalError::LookupError(symbol.to_string()))
    }

    /// Helper: Create a new scope with these arguments bound to it
    pub fn bind_args(&self, names: &[String], values: &SExpr) -> Self {
        let mut child_scope = self.child();
        zip(names.iter(), values.iter()).for_each(|(name, param)| {
            child_scope.set(name, param.clone());
        });
        child_scope
    }
}
