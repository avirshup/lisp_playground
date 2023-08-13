type Evaluator = dyn Fn(Vec<Expr>) -> Expr;

struct Function<'a> {
    fixed_args: u8,
    variadic: bool,
    eval: &'a Evaluator,
}

struct Scope {
    parent: Option<Rc<Scope>>,
    symbols: HashMap<String, Expr>,
}

/************\
|* builtins *|
\************/
fn builtin_lookup(key: &str) {
    match key {
        "+" => BuiltIn("+"),
    }
}
