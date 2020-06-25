use crate::commands::{self, prelude::*};
use clap::AppSettings;

pub fn build_cli() -> App {
    App::new("AMRAAM")
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
        .subcommands(commands::cli())
}
