use std::collections::HashMap;
use std::iter::Map;
use std::rc::Rc;

use super::Var;

pub type Mapping = HashMap<String, Var>;

/***
So, like, what is the type system here?
 - Trait-like.
 - Not duck-typed.
 - Strong, prolly. Or maybe strong-ish?
 - Allows delegation?
 - Static, maybe?

 At what level is it implemented?
 Is it operative at the lisp layer, or built on top of it?
 * Baked in advantages: makes it real, means I implement
    it in rust instead of lisp, which is a better way to do it
    than trying to write the language in itself I think.
    Can allow "Any" or whatever to allow for gradual or inferred
    or dynamic typing.
 Implemented in lisp: more flexible, can mess around with it.
    Can handle gradual typing. Types can be first class.

 DECISION: Build it in, write it in rust.
 This means that vars:
   - should have type info built-in always.
   - Types are not able to be treated as first-class objects
 */

#[derive(Debug, PartialEq)]
pub struct TypeInfo {
    name: String,
    // implementations: HashMap<String>,
}

pub struct Object {
    typeinfo: Rc<TypeInfo>,
    metadata: Mapping,
    data: Mapping,
}
