use anyhow::{Context, Result};
use nix::unistd::{seteuid, Uid};
use std::{
    fs::read_dir,
    path::Path,
    process::{exit, Command},
};

pub fn list_mods<P>(path: P) -> Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let mut mods = read_dir(path)
        .context("Could not read directory contents")?
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            Ok(entry
                .path()
                .file_name()
                .context("Path has no filename")?
                .to_str()
                .context("Path has invalid characters")?
                .to_owned())
        })
        .collect::<Result<Vec<String>>>()?;

    mods.sort();

    Ok(mods)
}

/// Escalate with sudo or set uid when not running with root permission.
///
/// This is equal to
/// [`sudo::escalate_if_needed`](https://docs.rs/sudo/0.3.1/sudo/fn.escalate_if_needed.html)
/// in function but preserves the `EDITOR` and `VISUAL` environment variables.
/// Additionally the result of seteuid is checked.
pub fn escalate_if_needed() -> Result<()> {
    use sudo::{check, RunningAs};

    let current = check();
    trace!("Running as {:?}", current);

    match current {
        RunningAs::Root => {
            trace!("already running as Root");
            return Ok(());
        }
        RunningAs::User => {
            debug!("Escalating privileges");
        }
        RunningAs::Suid => {
            trace!("setuid(0)");
            seteuid(Uid::from_raw(0)).context("Could not set effective userid")?;
            return Ok(());
        }
    }

    let args = std::env::args();
    let ecode = Command::new("/usr/bin/sudo")
        .arg("--preserve-env=EDITOR,VISUAL")
        .args(args)
        .spawn()
        .context("Failed to execute child")?
        .wait()
        .context("Failed to wait for child")?;

    if ecode.success() {
        exit(0);
    } else {
        exit(ecode.code().unwrap_or(1));
    }
}
