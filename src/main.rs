// mod builtins;
mod builtins;
mod ctypes;
mod eval;
mod expressions;
// mod functions;
mod parser;
mod procs;
mod scope;
mod token2expr;
mod tokenizer;

use std::rc::Rc;

use anyhow::Result;
use rustyline::history::DefaultHistory;
use rustyline::Editor;

use crate::eval::eval;
use crate::expressions::Expr;
use crate::scope::Scope;

fn main() -> Result<()> {
    // init scopes
    let root_scope = Rc::new(builtins::builtins());
    let mut repl_scope = Scope::child(root_scope);

    // start reading lines
    let mut rl = rl_editor()?;
    loop {
        // get user input
        let input = rl.readline(">> ");
        let Ok(input) = input else {
            continue;
        };
        if input == "exit" || input == "quit" {
            break;
        }

        // Read
        let s_exp = match parser::parse_text(&input) {
            Ok(s_exp) => s_exp,
            Err(err) => {
                println!("Parse error: {err}");
                continue;
            },
        };

        // Eval
        let result = match eval(Rc::new(Expr::SExpr(s_exp)), &mut repl_scope) {
            Ok(result) => result,
            Err(err) => {
                println!("Eval error: {err}");
                continue;
            },
        };

        // Print
        println!("{result}");
    }

    Ok(())
}

fn rl_editor() -> Result<Editor<(), DefaultHistory>> {
    let cfg = rustyline::Config::builder()
        .tab_stop(2)
        .auto_add_history(true)
        .bracketed_paste(true)
        .build();

    // let mut rl = rustyline::Editor::with_config(cfg.build())?;
    Ok(rustyline::DefaultEditor::with_config(cfg)?)
}
