#[macro_use]
extern crate log;

mod subcommands;

use anyhow::Result;
use clap::{App, Arg, SubCommand};

fn main() -> Result<()> {
    let matches = App::new("AMRAAM")
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
        .subcommand(SubCommand::with_name("init").about("Creates and installs a new server"))
        .subcommand(
            SubCommand::with_name("run")
                .about("runs the current arma installation")
                .arg(
                    Arg::with_name("option set")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("update")
                .about("Update or install the dedicated server or Steam Workshop mods")
                .subcommand(
                    SubCommand::with_name("server").about("Update or install the dedicated server"),
                )
                .subcommand(
                    SubCommand::with_name("mods")
                        .about("Update or install workshop mods")
                        .arg(
                            Arg::with_name("option set")
                                .required(true)
                                .help("Sets the option set to use when upating mods"),
                        ),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("init", Some(sub_matches)) => subcommands::init(sub_matches),
        ("run", Some(sub_matches)) => subcommands::run(sub_matches),
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}
