use crate::commands::prelude::*;

pub fn cli() -> App {
    SubCommand::with_name("mods")
        .about("Manage mods")
        .subcommands(vec![install::cli(), fix::cli(), list::cli()])
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let (cmd, sub_args) = args.subcommand();

    let f = match cmd {
        "fix" => fix::exec,
        "install" => install::exec,
        "list" => list::exec,
        _ => {
            cli().print_help()?;
            return Ok(());
        }
    };

    f(sub_args.context("Missing arguments")?)
}

pub mod fix;
pub mod install;
pub mod list;
