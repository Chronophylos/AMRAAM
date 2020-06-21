use amraam::Settings;
use anyhow::{bail, ensure, Context, Result};
use clap::ArgMatches;
use console::Term;
use fs_extra::dir::{copy_with_progress, CopyOptions, TransitProcess};
use indicatif::{HumanBytes, ProgressBar};
use std::{fs, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("Could not normalize path")]
    Canonicalize,
}

pub fn install(matches: &ArgMatches) -> Result<()> {
    let settings =
        Settings::from_path(matches.value_of("config")).context("Could not load settings")?;

    let path = matches.value_of("path").context("Missing path to mod")?;
    let mod_path = Path::new(path)
        .canonicalize()
        .context(InstallError::Canonicalize)?;

    ensure!(mod_path.is_dir(), "Mod path is not a directory");

    let server_path = settings
        .get_str("server.path")
        .context("Could not read server path from config")?
        .unwrap_or(String::from("./arma3"));

    let mods_path = Path::new(&server_path).join("mods");

    if !mods_path.exists() {
        fs::create_dir_all(&mods_path)?;
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
            //copy_inside: true,
            ..CopyOptions::new()
        }; //Initialize default values for CopyOptions

        let handle = |process_info: TransitProcess| {
            bar.tick();
            bar.set_message(&format!("Copied {}", HumanBytes(process_info.total_bytes)));
            fs_extra::dir::TransitProcessResult::ContinueOrAbort
        };

        copy_with_progress(&mod_path, &mods_path, &options, handle)
            .context("Could not copy files to mod dir")?;

        fs::rename(&mods_path.join(old_name), &target_path)?;
    } else {
        // move
        fs::rename(&mod_path, &target_path).context("Could not move mod to mods")?;
    }

    if matches.is_present("rename files") {
        let bar = ProgressBar::new_spinner();
        bar.enable_steady_tick(100);
        lowercase_all_files(target_path).context("Could not rename all files to lowercase")?;
        bar.finish();
    }

    Term::stdout().write_line(&format!("Sucessfully installed {}", &name))?;

    Ok(())
}

fn lowercase_all_files<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                lowercase_all_files(&path)?;
            }

            let name = &path.file_name().unwrap().to_str().unwrap().to_lowercase();
            fs::rename(&path, path.with_file_name(name))?;
        }
    }

    Ok(())
}
