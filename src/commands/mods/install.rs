use crate::commands::prelude::*;
use amraam::tools::{chmod, chown, lowercase};
use console::Term;
use fs_extra::dir::{copy_with_progress, CopyOptions};
use indicatif::{HumanBytes, ProgressBar};
use std::{fs, path::Path};
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
        .about("Install a mod")
        .long_about("Move a mod to the mods directory.")
        .args(&[
            Arg::with_name("path")
                .required(true)
                .takes_value(true)
                .help("The path to the mod you want to install"),
            Arg::with_name("name")
                .short("n")
                .long("name")
                .help(
                    "Set the mod name. If this option is not set the name is taken from the path.",
                )
                .takes_value(true),
            Arg::with_name("update")
                .short("u")
                .long("update")
                .help("Update mod by removing the old files first"),
            Arg::with_name("rename files")
                .short("l")
                .long("rename-files")
                .help("Rename all files to lowercase"),
            Arg::with_name("keep")
                .short("k")
                .long("keep")
                .help("Keep the original files"),
            Arg::with_name("force")
                .short("f")
                .long("force")
                .help("Force overwriting existing files"),
        ])
}

pub fn exec(matches: &ArgMatches) -> Result<()> {
    match sudo::escalate_if_needed() {
        Ok(_) => {}
        Err(err) => bail!("Could not escalate with sudo: {}", err),
    };

    let settings =
        Settings::from_path(matches.value_of("config")).context("Could not load settings")?;

    let path = matches.value_of("path").context("Missing path to mod")?;
    let mod_path = Path::new(path)
        .canonicalize()
        .context(InstallError::Canonicalize)?;

    ensure!(mod_path.is_dir(), "Mod path is not a directory");

    let server_path = settings
        .get_server_path()
        .context("Could not get server path from config")?;

    let mods_path = Path::new(&server_path).join("mods");

    let user = settings
        .get_str("server.user")
        .context("Could not get server user from config")?
        .context("Missing config key server.user")?;

    if !mods_path.exists() {
        fs::create_dir_all(&mods_path)?;
        chown(&mods_path, &user, true).context(InstallError::Chown)?;
        chmod(&mods_path, 0o755, 0o644, true).context(InstallError::Chmod)?;
    }

    let mods_path = mods_path
        .canonicalize()
        .context(InstallError::Canonicalize)?;

    let old_name = mod_path
        .file_name()
        .context("Path does not point to a file or directory")?
        .to_str()
        .context("Could not convert os string to string")?;

    let name = match matches.value_of("name") {
        Some(n) => n.to_owned(),
        None => old_name.to_lowercase().replace(" ", "_"),
    };

    let target_path = mods_path.join(&name);

    if matches.is_present("update") {
        if target_path.is_dir() {
            fs::remove_dir_all(&target_path).context("Could not remove old mod")?;
        }
    }

    if !target_path.is_dir() {
        fs::create_dir(&target_path).context("Could not create mod directory in mods")?;
    } else {
        if !matches.is_present("force") {
            bail!("Mod directory already exists");
        }
    }

    if matches.is_present("keep") {
        // copy
        let bar = ProgressBar::new_spinner();
        let options = CopyOptions {
            overwrite: matches.is_present("force"),
            ..CopyOptions::new()
        };

        copy_with_progress(&mod_path, &mods_path, &options, |process_info| {
            bar.tick();
            bar.set_message(&format!("Copied {}", HumanBytes(process_info.total_bytes)));
            fs_extra::dir::TransitProcessResult::ContinueOrAbort
        })
        .context("Could not copy files to mod dir")?;
        bar.finish_with_message("Finished copying files to mod dir");

        fs::rename(&mods_path.join(old_name), &target_path)?;
    } else {
        // move
        fs::rename(&mod_path, &target_path).context("Could not move mod to mods")?;
    }

    if matches.is_present("rename files") {
        lowercase(&target_path, true).context("Could not rename files to lowercase")?;
    }

    chown(&target_path, &user, true).context(InstallError::Chown)?;
    chmod(&target_path, 0o755, 0o644, true).context(InstallError::Chmod)?;

    Term::stdout().write_line(&format!("Sucessfully installed {}", &name))?;

    Ok(())
}
