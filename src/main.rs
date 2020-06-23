#[macro_use]
extern crate log;

mod commands;

use anyhow::{bail, Context, Result};
use clap::{App, AppSettings, Arg};
use console::{style, Term};

fn main() -> Result<()> {
    let mut app = App::new("AMRAAM")
        .version("0.1.0")
        .author("Nikolai Zimmermann")
        .about("Manage ArmA 3 Dedicated Server installations")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Sets a custom config location (default: amraam.toml)")
                .takes_value(true),
        )
        .settings(&[AppSettings::VersionlessSubcommands])
        .subcommands(commands::cli());

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
