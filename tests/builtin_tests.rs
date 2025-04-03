use lisp_playground::ast::{Expr, Value, Var};
use lisp_playground::parser::parse_text;
use lisp_playground::{builtins, eval};

fn parse_and_eval(s: &str) -> Var {
    let parsed: Var = parse_text(s).unwrap().into();

    let root_scope = builtins();
    let mut eval_scope = root_scope.child();
    println!("Eval: {parsed}");
    eval(&parsed, &mut eval_scope).unwrap()
}

fn assert_expressions_equal(lhs: &str, rhs: &str) {
    let lval = parse_and_eval(lhs);
    let rval = parse_and_eval(rhs);
    assert_eq!(lval.as_ref(), rval.as_ref());
}

// fn assert_true(exp: &str) {
//     let result = parse_and_eval(exp);
//     if let Expr::Value(Value::Bool(val)) = result.as_ref() {
//         assert!(val)
//     } else {
//         panic!("Not a bool: {result}")
//     }
// }

fn assert_var_eq<T>(expected: T, actual: &Var)
where
    T: Into<Var> + PartialEq,
{
    assert_eq!(expected.into(), actual.clone());
}

//*** BASE TESTS ***//
// These are a little verbose to ensure that we can
// rely on the test evaluation machinery in later, more concise tests
#[test]
fn test_quote() {
    let result = parse_and_eval("(quote 'hello' 'world')");
    let sexp = result.expect_sexp_with_len(2).unwrap();

    assert_var_eq(
        Value::Str("hello".to_string()),
        sexp.get(0).unwrap(),
    );
    assert_var_eq(
        Value::Str("world".to_string()),
        sexp.get(1).unwrap(),
    );

    // meta-test sanity check
    assert_expressions_equal(
        "(quote 'hello' 'world')",
        "(quote 'hello' 'world')",
    );
}

#[test]
fn test_identity() {
    // meta-test sanity check
    assert_expressions_equal("(quote 3)", "(echo (quote 3))");
}

#[test]
fn test_first() {
    let result = parse_and_eval("(first (quote 1 2))");
    assert_var_eq(Value::Int(1), &result);
}

#[test]
fn test_rest() {
    assert_expressions_equal(
        "(rest (quote 1 2 'hi' ('yo' 'yo')))",
        "(quote 2 'hi' ('yo' 'yo'))",
    );
}

#[test]
fn test_concat() {
    assert_expressions_equal(
        "(concat (quote 1 2) (quote 3 4))",
        "(quote 1 2 3 4)",
    );
}

#[test]
fn test_len() {
    assert_expressions_equal(
        "(len (concat (quote 1 2) (quote 3 4)))",
        "(echo 4)",
    );
}
