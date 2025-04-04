# a playground for messing around with lisp and rust

A bog-standard hobby S-expression language with a treewalk intepreter. Use `cargo run` to launch an interactive
interpreter.

The most
interesting thing this does is "greedy binding" - functions/closures
capture _values_ from their enclosing scopes _eagerly_ when possible (which
I think makes this not a real lisp). This is accomplished by each special form having its own special method to capture
the variables it needs from its enclosing scope (see, e.g., the `bind_outer_scope` method for the [
`DefineForm`](src/builtins/special_forms.rs))
