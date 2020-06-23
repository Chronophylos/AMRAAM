mod prelude;

use clap::ArgMatches;
use prelude::*;

pub fn cli() -> Vec<App> {
    vec![mods::cli(), init::cli(), run::cli(), missions::cli()]
}

pub fn exec(cmd: &str) -> Option<fn(&ArgMatches<'_>) -> Result<()>> {
    let f = match cmd {
        "init" => init::exec,
        "run" => run::exec,
        "mods" => mods::exec,
        "missions" => missions::exec,
        _ => return None,
    };
    Some(f)
}

pub mod init;
pub mod missions;
pub mod mods;
pub mod run;
