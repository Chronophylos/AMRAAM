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
                .about("Runs the current arma installation")
                .arg(Arg::with_name("option set").takes_value(true)),
        )
        .subcommand(
            SubCommand::with_name("mods")
                .about("Manage mods")
                .subcommand(
                    SubCommand::with_name("install")
                        .about("Install a mod")
                        .long_about("Move a mod to the mods directory.")
                        .arg(
                            Arg::with_name("path")
                                .required(true)
                                .takes_value(true)
                                .help("The path to the mod you want to install"),
                        )
                        .arg(
                            Arg::with_name("name")
                                .long("name")
                                .help("Set the mod name. If this option is not set the name is taken from the path.")
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("update")
                                .short("u")
                                .long("update")
                                .help("Update mod by removing the old files first")
                        )
                        .arg(
                            Arg::with_name("rename files")
                                .short("l")
                                .long("rename-files")
                                .help("Rename all files to lowercase"),
                        )
                        .arg(
                            Arg::with_name("keep")
                                .short("k")
                                .long("keep")
                                .help("Keep the original files"),
                        )
                        .arg(
                            Arg::with_name("force")
                                .short("f")
                                .long("force")
                                .help("Force overwriting existing files")
                        ),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("init", Some(sub_matches)) => subcommands::init(sub_matches),
        ("run", Some(sub_matches)) => subcommands::run(sub_matches),
        ("mods", Some(sub_matches)) => match sub_matches.subcommand() {
            ("install", Some(sub_sub_matches)) => subcommands::mods::install(sub_sub_matches),
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}
