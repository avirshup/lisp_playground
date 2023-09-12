use anyhow::Result;
use rustyline::config::Configurer;

fn main() {}

fn rl_editor() -> Result<rustyline::DefaultEditor> {
    let cfg = rustyline::Config::builder()
        .tab_stop(4)
        .auto_add_history(true)
        .bracketed_paste(true)
        .build();

    // let mut rl = rustyline::Editor::with_config(cfg.build())?;
    rustyline::DefaultEditor::with_config(cfg).map_err(anyhow::Error::from)
}
