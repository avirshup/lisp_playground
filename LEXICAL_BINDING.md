Notes on semantics and implementation of name binding to outer scopes.

# The problem

## Rules
We impose 3 rules
 1) **no expressions may be evaluated** while capturing - this is a lexical binding.
 2) variables must be captured from the enclosing scope
    when the closure is created, not when it is called.
 3) Similarly, undefined variables in closures **must cause 
      errors** when the closure is created, not when it is called.

## Goals
These can't be rules because they aren't compatible with the rules;
we aren't going to be able to exactly satisfy them.

### Goal A: don't capture unnecessary variables
We want to _only capture necessary variables_,
in order to not keep lots of uneccesary references and scopes around. However, if necessary, we can allow capturing a superset of them.

#### Yes, it's possible to capture a superset of outer variables
This follows from the definition of lexical binding: a variable may
be bound if and only if it appears lexically within the enclosing
scope of the closure's definition. So if we just capture all symbols
that appear in our expression, we've captured everything we need
by definition.

#### No: this violates rule 3.
If we can't tell whether a variable needs to be captured or not,
that means we don't know if it's undefined or not.

### Goal B: definition-time error _only_ for undefined symbols.
For a static language, this is necessary.  However, if necessary,
we allow throwing definition-time error if 
_we can't tell_ whether a variable will be undefined or not.

## Complications
What makes this hard? Nested special forms and shadowed variables.

If there are inner symbols that **shadow** names in the
outer scope, the shadowed variables may be captured anyway.
How?

### Situation 1: symbols defined _inside_ the closure's scope

Symbols can, of course, be bound _within_ the scope of the closure.
Let's take the case of curied addition: `(lambda (f x) (lambda (f y) (+ x y)))`.  When capturing here, we need to know _not_ to raise an error when we encounter `x` or `y`. To solve this, we weed to have a
special-form-aware capture analysis that can keep track of the scope
inside the closure.

Such an analysis routine will also understand variable shadowing.


### Situation 2: Identifying special forms

The conceit of "special-form-aware capture analysis" _presumes that
we can identify special forms_. But if users redefine the special form
inside the closure, where by rule 1, we're not allowed to evaluate anything? Thus, we need to impose **Constraint 1**: no special form aliasing. (see below)

## Constraints
Due to the complications, we arrive at some constraints
to impose on the language.

### Constraint 1: no special form aliasing
Raw special forms (i.e, `lambda` itself, not calls to `lambda`)
must not be aliased, shadowed, passed to, or returned from functions.

Examples:
**No returning**: It is a syntax error to run `(def x define)`.
**No passing to specials**: It is a syntax error to define `(quote define)`.
**No passing to functions**:: It is a syntax error to run `(define (mycallable x) (...)); (mycallable lambda)`.
**No shadowing**: It is a syntax error to run `(def lambda 3)`.

NOTE: It's fine, of course, to _call_ a special form within a callable.
And it's fine to put a _call_ to a special form in the argument to a function (not sure about specials)?

#### Also
If I make other, non-sexpr-based syntaxes for this language,
the special forms mapped to special syntactic constructs that
don't have an explicit namespace symbol associated with them.

#### Maybe it's illegal to quote special forms at all?
Dunno, `quote` doesn't exist yet, should think on it.

### Constraint 2: special forms need to do the binding for us.
Rather than code the syntax of all the special forms into our capture 
analysis, whenever we encounter a nested special form while defining a
closure, we'll call its `bind_outer_scope` method that will do the
binding for us.


#### Implementation
`bind_outer_scope` methods right now create new children of the outer scope to be passed back to the variable capturer. Into this scope, they set any symbols that _will_ be defined at runtime within the closure as
`Symbol(s) -> Symbol(s)`. This tautological mapping is the signal to _not_
capture the symbols in the closure's scope (see `crate::closures::capture_symbol_reference`)

# (original notes) Other languages 
Kinda shocking how many different ways there are of doing this.

### Python, JavaScript, Racket, Swift
Python does _the latest possible_ binding _by name_ - the name is looked up in its outer _scope whenever the function is called.

Racket and JavaScript do the same.

```python
>>> def add_outer(n):
...    n + outer
>>>
>>> outer = 3
>>> add_outer(3)
5
>>> outer = 100; add_outer(3)
103
>>> del outer; add_outer(3)
Traceback [...]
NameError: name 'outer' is not defined. Did you mean: 'iter'?
```


### Rust
Rust doesn't really _bind_ like the others - it either _moves_ or _borrows_ according to its normal rules.

Also note that, while rust _can_ have named functions inside other
functions, [they can't refer to variables in the enclosing _scope.](https://doc.rust-lang.org/error_codes/E0434.html)

```rust
fn main() {
    let outer = 3;
    fn add_outer(n: isize) -> isize {
        n +&outer  // ERROR DOES NOT COMPILE
    }
}
```

### Haskell
At least in GHCI, haskell looks up and binds _values_ immediately. Renaming `outer` won't change add_outer:

```haskell
ghci> outer = 3
ghci> add_outer n = outer + 3
ghci> add_outer 3
6
ghci> outer = 100
ghci> add_outer 3
6
```

### Clojure and Racket
Clojure seems to capture _by mutable reference_.

It will error if symbol doesn't exist:
```clojure
=> (defn add_outer [x] (+ x outer))
Syntax error compiling at (REPL:1:21).
Unable to resolve symbol: outer in this context
```

The symbol can be mutated, and it will change, but if deleted from the namespace it will still exist (and can't be changed):
```clojure
user=> (def outer 3)
#'user/outer
user=> (defn add-outer [x] (+ x outer))
#'user/add-outer
user=> (add-outer 3)
6

user=> (def outer 6)
#'user/outer
user=> (add-outer 3)
9

user=> (ns-unmap (find-ns 'user) 'outer) ; deletes it
nil
user=> outer
Syntax error compiling at (REPL:0:0).
Unable to resolve symbol: outer in this context
user=> (add_outer 3)
9
user=> (def outer 100)
#'user/outer
user=> (add_outer 3)
9
```


### Swift - late binding like python and friends!?
Not what I expected at all - it would seem to have a runtime 
... or at the very least a very explicit notion of scopes?
(Or something else I haven't considered) 

```swift
func run() -> Void {
    func add_outer(n: Int) -> Int {
        return n + outer
    }

    var outer = 3
    print("Hello", add_outer(n: 3)) // 7
    
    outer = 10
    print("Hello", add_outer(n: 3)) // prints 13
}

run()
```

Note that this _doesn't work:
```swift
func run() -> Void {
    func add_outer(n: Int) -> Int {
        return n + outer
    }
    print("Hello", add_outer(n: 3)) // ERROR DOES NOT COMPILE
    var outer = 3
}
```


# Test cases

## Standard enclosing _scope capture

### The basics
```lisp
(assert-fails 
    (define (add-outer x) (+ outer 3)
    ))
```

```lisp
(define outer 3)
(define (add-outer x) (+ outer 3))
(assert-eq (add-outer 3) 6)

(define outer 100)
(assert-eq (add-outer 3) 6)
(assert-eq outer 100)
```

Args should override outer _scope
```lisp
(define x 10)

;; Note that define still defines in the outermost _scope
(define (is10? x)
    (= x 10))
    
(assert-not (is10? 5))
```

Args in nested functions should also override outer _scope:
```lisp
(define y 10)
(define nine 9)

(define (is10? x) (
    (lambda (y) (== 9 y)) (+ x -1)
)
```
How to implement this? Easier with late binding, because you don't need
anything special to evaluate the damn things.

But otherwise ... the `define` and `lambda` special forms need to be
aware of nested function definitions when preparing the _scope.

What about capturing a lambda from within a lambda?

```lisp
(define x 100)
(define y 200)
(define z 1000)

;; should capture the lambda's x, but outer z
(
    define (hi x) (
        lambda (y) (+ (+ x y) z)
    )
)

(assert-eq ((hi 3) 5) 1008)
```

Given my implementation, this one might be a problem.
Does the outer scoped variable correctly make it through
sibling statements, one of which uses it as a parameter,
but the other one closes over it?

```lisp
(define y 200)

( define (hi x)
  ( if (> x 10)
       ((lambda (y) (* y 1000)) x)
       ((lambda (z) (+ y z)) x)
))
  
(assert-eq (hi 5) 205)
(assert-eq (hi 20) 20000)
```



## Scoping rules
Unlike lisp and clojure, `define` and `defvar` _do not_ set global variables -
they set them in the scope in which they're called. The following things set a new scope:

 - the body of a `lambda`
 - a `do`





Note that`let` does _not_.



## Implementation

All special forms must provide a
`SpecialForm::bind_outer_scope(sexpr: &Sexpr, scope: &Scope) -> Scope` method,
which, given the form's s-expression and its enclosing scope,
return all variables they are going to need
from the outer _scope. This method _must not_ evaluate anything.

### Hygiene rules
Consider an AST of the form `(<outer AST> (my-special-form <inner AST>))`. The symbols in the returned scope must be a _subset of the intersection_ of all symbols in the outer AST and inner AST.

Not sure about:
 - this is _necessary_ to enforce lexical scoping, but not sure if _sufficient_?
 - is this "lexical scoping" _equivalent_ to "macro hygiene"?

## Other notes
### Special forms need special treatment
Lambdas are going to need to treat other special forms specially ...
and maybe vice versa. ALL SPECIAL FORMS MUST TELL LAMBDA WHEN THEY
CREATE A NEW SCOPE OR DEFINE VARIABLES.

Oh wait, we can do even better.


### Hygiene
Get-out-of-jail-free card: this is LEXICAL _scope.
We only need to worry about Symbols that actually appear within the AST!

But actually, we need to _ensure_ that this only affects macros that appear within the AST - Macros could potentially introduce
variables that would get bound to the _scope when this gets evaluated.

So, let's forbid that - special forms must be hygienic.

Maybe we need to bind _individual symbols_ in the AST.

#### Rambling
Walkthrough below. Some keys here:
 1) The lambda special form need to treat lambdas specially.
 2) Variables are captured within an orphan _scope assigned to the
    lambda; unlike my first attempt, we don't replace any symbols
    in the expression. This means that mistakes won't actually
    affect the final evaluation!
 3) Alternatively, much simpler would be capturing any variables
    that share names from enclosing _scope, even if they turn out
    not to be needed during evaluation. Similarly to if we
    just kept a reference to the _scope around, this could keep
    objects alive longer than necessary! 

Two alternatives:
 1) evaluate sub-lambdas while evaluating lambdas. This is not ideal,
    we should not be evaluating _anything_ while evaluating lambdas.
 2) 

(Or, maybe, shadowing should be illegal within closures ...)

#### Too much walkthrough
When it evaluates the function def, it should:
1) invoke the `define` special form
2) transform to `(define hi (lambda (x) (lambda (y) (+ x y))))`
3) invoke `define` again to set `hi` in _scope, and start evaluating the value.
4) Invoke the outer `lambda (x)` special form. It _two_ new scopes: 
  - The **OUTER LOOKUP** _scope, which is a child of the outer _scope. It has x->Symbol("x").  This is temporary and will be droppet after `lambda` is done.
  - The **OUTER CAPTURE** _scope, which does not have a parent. This will be permanently assigned to the lambda expression.
5) The lambda form finds the inner lambda; this starts a recursive lambda form running. In this case, its lookup _scope is a child of the lookup _scope from, with `y -> Symbol(y)` 
6) It processes the symbols in lambda as follows:
   A) The first argument to lambda, `(y)` is _ignored_.
   B) It finds `Symbol('x')` in the _outer lookup scope_ then looks it up and captures it as `Symbol(x)` in the inner capture _scope.
   C) Same thing for `y`, except it's found in the inner lookup _scope.
   D) `z` is located in the _enclosing scope_ and copied to the inner capture _scope.
7) The second lambda is done. It's saved, with its lookup _scope, as an `Expr::Function(Function::Lambda)` kind of thing.
8) The first lambda finishes too.


