#[macro_use]
extern crate log;

mod cli;
mod commands;

use anyhow::{Context, Result};
use console::{style, Term};

fn main() -> Result<()> {
    let mut app = cli::build_cli();

    let args = app.clone().get_matches_safe()?;
    let (cmd, sub_args) = args.subcommand();
    let stdout = Term::stdout();

    if let Some(exec) = commands::exec(cmd) {
        match exec(sub_args.context("Missing arguments")?) {
            Ok(_) => {}
            Err(err) => {
                stdout.write_line(&format!("{}: {:?}", style("Error").red(), err))?;
            }
        }
    } else {
        app.print_help()?;
    }
    Ok(())
}
