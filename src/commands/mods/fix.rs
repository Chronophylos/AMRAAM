use crate::commands::prelude::*;
use amraam::{
    tools::{chmod, chown, lowercase},
    Settings,
};
use std::path::Path;

pub fn cli() -> App {
    SubCommand::with_name("fix")
        .about("Fix a mod installation")
        .arg(
            Arg::with_name("name")
                .required(true)
                .takes_value(true)
                .help("The name of the mod"),
        )
        .arg(
            Arg::with_name("rename files")
                .short("l")
                .long("rename-files")
                .help("Rename all files to lowercase"),
        )
        .arg(
            Arg::with_name("permissions")
                .short("p")
                .long("permissions")
                .help("Fix permissions"),
        )
        .arg(
            Arg::with_name("owner")
                .short("o")
                .long("owner")
                .help("Fix owner"),
        )
}

pub fn exec(matches: &ArgMatches) -> Result<()> {
    match sudo::escalate_if_needed() {
        Ok(_) => {}
        Err(err) => bail!("Could not escalate with sudo: {}", err),
    };

    let settings =
        Settings::from_path(matches.value_of("config")).context("Could not load settings")?;

    let server_path = settings
        .get_server_path()
        .context("Could not get server path from config")?;

    let mods_path = Path::new(&server_path).join("mods");

    let mod_name = matches
        .value_of("name")
        .context("Could not get mod name from arguments")?;

    let mod_path = mods_path.join(mod_name);

    ensure!(mod_path.exists(), "Mod not found");

    if matches.is_present("rename files") {
        lowercase(&mod_path, true).context("Could not rename files to lowercase")?;
    }

    if matches.is_present("permissions") {
        chmod(&mod_path, 0o755, 0o644, true).context("Could not change permissions of mod")?;
    }

    if matches.is_present("owner") {
        let name = settings
            .get_str("server.user")
            .context("Could not get server user from config")?
            .context("Missing key `server.user`")?;

        chown(&mod_path, &name, true).context("Could not change ownernship of mod")?
    }

    Ok(())
}
