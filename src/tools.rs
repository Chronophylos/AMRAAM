use anyhow::{Context, Result};
use indicatif::ProgressBar;
use nix::unistd::{self, User};
use std::{
    fs::{self, set_permissions, Permissions},
    os::unix::fs::PermissionsExt,
    path::Path,
};
use walkdir::WalkDir;

pub fn chown<P>(path: P, username: &str, recursive: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let user = User::from_name(username)
        .context("Could not get user by name")?
        .context("Could not find user")?;
    let path = path.as_ref();

    if recursive {
        let bar = ProgressBar::new_spinner();

        let iter = WalkDir::new(&path).follow_links(true);

        for entry in bar.wrap_iter(iter.into_iter()) {
            let path = entry.context("Could not get path")?.into_path();

            debug!("Changing ownership of {} to {}", path.display(), user.name);
            bar.set_message(&format!("{}", path.display()));

            unistd::chown(&path, Some(user.uid), Some(user.gid))
                .context("Could not change ownership of file")?;
        }

        bar.finish_with_message("Finished changing ownership");
    } else {
        debug!("Changing ownership of {} to {}", path.display(), user.name);

        unistd::chown(path, Some(user.uid), Some(user.gid))
            .context("Could not change ownership of file")?;
    }

    Ok(())
}

pub fn chmod<P>(path: P, directory_mode: u32, file_mode: u32, recursive: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let dir_perms = Permissions::from_mode(directory_mode);
    let file_perms = Permissions::from_mode(file_mode);

    if recursive {
        let bar = ProgressBar::new_spinner();

        let iter = WalkDir::new(&path).follow_links(true);

        for entry in bar.wrap_iter(iter.into_iter()) {
            let path = entry.context("Could not get path")?.into_path();

            let perms = if path.is_file() {
                &file_perms
            } else {
                &dir_perms
            };

            debug!(
                "Changing permissions of {} to {:o}",
                path.display(),
                perms.mode()
            );
            bar.set_message(&format!("{}", path.display()));

            set_permissions(&path, perms.clone()).context("Could not set permission of path")?;
        }

        bar.finish_with_message("Finished changing permissions");
    } else {
        let perms = if path.is_file() {
            file_perms
        } else {
            dir_perms
        };

        debug!(
            "Changing permissions of {} to {:o}",
            path.display(),
            perms.mode()
        );

        set_permissions(&path, perms).context("Could not set permission of path")?;
    }

    Ok(())
}

pub fn lowercase<P>(path: P, recursive: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if recursive {
        let bar = ProgressBar::new_spinner();

        let iter = WalkDir::new(&path).follow_links(true);

        for entry in bar.wrap_iter(iter.into_iter()) {
            let path = entry.context("Could not get path")?.into_path();
            let new_path = path
                .to_str()
                .context("Path is not valid UTF-8")?
                .to_lowercase();

            debug!("Renaming {} to {}", path.display(), new_path);
            bar.set_message(&format!("{}", path.display()));

            fs::rename(path, new_path).context("Could not rename path")?;
        }

        bar.finish_with_message("Finished renaming files");
    } else {
        let new_path = path
            .to_str()
            .context("Path is not valid UTF-8")?
            .to_lowercase();

        debug!("Renaming {} to {}", path.display(), new_path);

        fs::rename(path, new_path).context("Could not rename path")?;
    }

    Ok(())
}
