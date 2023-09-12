// mod builtins;
mod eval;
mod expressions;
mod procs;
mod scope;
mod token2expr;
mod tokenizer;

use std::io;
use std::io::Read;
use std::rc::Rc;

use anyhow::Result;
use eval::eval;
use expressions::{read_tokens, Expr, SExpr};
use scope::Scope;

fn main() {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .unwrap();

    let parse_result = parse(&buffer);
    println!("AST: {parse_result:#?}");

    if let Ok(expr) = parse_result {
        let expr_ptr = Rc::new(Expr::SExpr(expr));
        let mut scope = Scope::empty();
        let eval_result = eval(expr_ptr, &mut scope);
        println!("Result: {eval_result:#?}");
    };
}

fn parse(s: &str) -> Result<SExpr> {
    let tokens = tokenizer::tokenize(s);
    read_tokens(&mut tokens.iter())
}
