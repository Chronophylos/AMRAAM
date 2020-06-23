use anyhow::{Context, Result};
use std::{fs::read_dir, path::Path};

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
