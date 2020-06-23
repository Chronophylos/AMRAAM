use crate::commands::prelude::*;
use amraam::tools::{chmod, chown};
use console::Term;
use std::{
    fs::{create_dir_all, rename},
    path::Path,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("Could not normalize path")]
    Canonicalize,

    #[error("Could not change ownership of path")]
    Chown,

    #[error("Could not set permission of path")]
    Chmod,
}

pub fn cli() -> App {
    SubCommand::with_name("install")
        .about("Install a mission")
        .long_about("Move or copy a mission into the mpmissions directory.")
        .args(&[Arg::with_name("path")
            .required(true)
            .takes_value(true)
            .help("The path to the mission you want to install")])
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    match sudo::escalate_if_needed() {
        Ok(_) => {}
        Err(err) => bail!("Could not escalate with sudo: {}", err),
    };

    let settings =
        Settings::from_path(args.value_of("config")).context("Could not load settings")?;

    let path = args.value_of("path").context("Missing path to mission")?;
    let mission_path = Path::new(path)
        .canonicalize()
        .context(InstallError::Canonicalize)?;

    ensure!(mission_path.is_dir(), "Mod path is not a directory");

    let server_path = settings
        .get_server_path()
        .context("Could not get server path from config")?;

    let mpmissions_path = Path::new(&server_path).join("mpmissions");

    let user = settings
        .get_str("server.user")
        .context("Could not get server user from config")?
        .context("Missing config key server.user")?;

    if !mpmissions_path.exists() {
        create_dir_all(&mpmissions_path).context("Could not create mpmissions directory")?;
        chown(&mpmissions_path, &user, true).context(InstallError::Chown)?;
        chmod(&mpmissions_path, 0o755, 0o644, true).context(InstallError::Chmod)?;
    }

    let name = mission_path
        .file_name()
        .context("Path does not point to a file or directory")?;

    let target_path = mpmissions_path.join(name);

    rename(&mission_path, &target_path).context("Could not move mission to mpmissions")?;

    chown(&target_path, &user, true).context(InstallError::Chown)?;
    chmod(&target_path, 0o755, 0o644, true).context(InstallError::Chmod)?;

    Term::stdout().write_line(&format!(
        "Sucessfully installed {}",
        &name.to_string_lossy()
    ))?;

    Ok(())
}
