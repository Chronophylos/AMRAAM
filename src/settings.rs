use anyhow::Result;
use config::{Config, Environment, File};
use std::path::Path;

pub fn load_settings<P>(path: Option<P>) -> Result<Config>
where
    P: AsRef<Path>,
{
    let mut settings = Config::default();

    if let Some(path) = path {
        let file: File<_> = path.as_ref().into();
        settings.merge(file)?
    } else {
        // Add in `./amraam.toml`
        settings.merge(File::with_name("amraam"))?
    }
    // Add in settings from the environment (with a prefix of APP)
    // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
    .merge(Environment::with_prefix("AMRAAM"))?;

    Ok(settings)
}
