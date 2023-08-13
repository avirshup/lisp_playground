mod expressions;
mod token2expr;
mod tokenizer;

use std::io;
use std::io::Read;

use anyhow::Result;
use expressions::{read_tokens, SExpr};

fn main() {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .unwrap();

    let result = parse(&buffer);

    println!("{result:#?}");
}

fn parse(s: &str) -> Result<SExpr> {
    let tokens = tokenizer::tokenize(s);
    read_tokens(&mut tokens.iter())
}
