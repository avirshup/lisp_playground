use std::rc::Rc;

use anyhow::Result;
use rustyline::Editor;
use rustyline::history::DefaultHistory;

use crate::ast::Var;
use crate::{ast, builtins, eval, parser};

/// The repl
pub fn run() -> Result<()> {
    // init scopes
    let root_scope = builtins();
    let mut repl_scope = root_scope.child();

    // start reading lines
    let mut rl = rl_editor()?;
    loop {
        // Prompt
        let input = rl.readline(">> ");
        let Ok(input) = input else {
            continue;
        };

        // [R]ead
        if input == "exit" || input == "quit" {
            break;
        }
        // TODO: Ctrl-C and Ctrl-D
        // TODO: autocomplete
        let s_exp = match parser::parse_text(&input) {
            Ok(s_exp) => s_exp,
            Err(err) => {
                println!("Parse error: {err}");
                continue;
            },
        };

        // [E]val
        let result = match eval(
            &Var::new(ast::Expr::SExpr(s_exp)),
            &mut repl_scope,
        ) {
            Ok(result) => result,
            Err(err) => {
                println!("Eval error: {err}");
                continue;
            },
        };

        // [P]rint
        println!("{result}");
    } // [L]oop

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
