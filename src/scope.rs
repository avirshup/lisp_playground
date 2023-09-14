use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Expr;

// Question: who owns a child scope?

pub struct Scope {
    // how will the `set` implementation work?
    //      are we able to satisfy the borrow checker
    //      that a scope can't be mutated while it has
    //      children? Is that even true? (it feels truthy ¯\_(ツ)_/¯)
    //      Or, do we need interior mutability?
    parent: Option<Rc<Scope>>,

    // do we really need `Rc`s to exprs here?
    //      Even if we _do_ use ref counting, it might make sense
    //      to put it to the `Expr` variant definitions themselves ...
    symbols: HashMap<String, Rc<Expr>>,
}

impl Scope {
    pub fn empty() -> Self {
        Scope {
            parent: None,
            symbols: HashMap::new(),
        }
    }

    pub fn child(parent: Rc<Self>) -> Self {
        Scope {
            parent: Some(parent),
            symbols: HashMap::new(),
        }
    }

    pub fn lookup(&self, symbol: &str) -> Option<Rc<Expr>> {
        self.symbols
            .get(symbol)
            .cloned()
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|parent| parent.lookup(symbol))
            })
    }

    pub fn set(&mut self, key: &str, val: Rc<Expr>) {
        self.symbols
            .insert(key.to_string(), val);
    }
}
//
// /************\
// |* builtins *|
// \************/
// fn builtin_lookup(key: &str) {
//     match key {
//         "+" => BuiltIn("+"),
//     }
// }
