use crate::commands::prelude::*;
use amraam::{util::list_mods, Settings};
use console::Term;
use std::path::Path;

pub fn cli() -> App {
    SubCommand::with_name("list").about("List all installed mods")
}

pub fn exec(matches: &ArgMatches) -> Result<()> {
    let settings =
        Settings::from_path(matches.value_of("config")).context("Could not load settings")?;
    let term = Term::buffered_stdout();

    let server_path = settings
        .get_server_path()
        .context("Could not get server path from config")?;

    let mods_path = Path::new(&server_path).join("mods");

    term.write_line(&format!("Installed mods in {}:", mods_path.display()))
        .context("Could not write header line on terminal")?;

    for mod_name in list_mods(&mods_path).context("Could not list mods")? {
        term.write_line(&format!(" {}", mod_name))
            .context("Could not write line on terminal")?;
    }

    term.flush().context("Could not flush terminal")?;

    Ok(())
}
