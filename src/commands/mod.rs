use clap::ArgMatches;
use prelude::*;

pub fn cli() -> Vec<App> {
    vec![
        generate::cli(),
        init::cli(),
        missions::cli(),
        mods::cli(),
        run::cli(),
        completions::cli(),
    ]
}

pub fn exec(cmd: &str) -> Option<fn(&ArgMatches<'_>) -> Result<()>> {
    let f = match cmd {
        "generate" => generate::exec,
        "init" => init::exec,
        "missions" => missions::exec,
        "mods" => mods::exec,
        "run" => run::exec,
        "generate-completions" => completions::exec,
        _ => return None,
    };
    Some(f)
}

pub mod completions;
pub mod generate;
pub mod init;
pub mod missions;
pub mod mods;
pub mod prelude;
pub mod run;
