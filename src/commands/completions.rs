use crate::commands::prelude::*;
use clap::Shell;
use std::{io, str::FromStr};

pub fn cli() -> App {
    SubCommand::with_name("generate-completions")
        .about("Generate shell completions")
        .arg(
            Arg::with_name("type")
                .possible_values(&Shell::variants())
                .default_value("bash"),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let shell = Shell::from_str(args.value_of("type").context("Missing arg `type`")?).unwrap();

    crate::cli::build_cli().gen_completions_to("AMRAAM", shell, &mut io::stdout());

    Ok(())
}
