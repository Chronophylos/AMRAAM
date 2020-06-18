use amraam::config::OptionSet;
use amraam::settings::load_settings;
use anyhow::{Context, Result};
use clap::ArgMatches;
use std::process::Command;

macro_rules! arg {
    ($command:expr, $option:expr, $arg:expr) => {
        if let Some(name) = $option {
            $command.arg(&format!("-{}={}", $arg, name));
        }
    };
}

macro_rules! arg_bool {
    ($command:expr, $option:expr, $arg:expr) => {
        if let Some(true) = $option {
            $command.arg(&format!("-{}", $arg));
        }
    };
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    let settings = load_settings(matches.value_of("config"))?;

    let options_name = matches.value_of("option set").unwrap();

    // TODO: handle case when global is not set
    let global_options: OptionSet = settings
        .get("options.global")
        .context("Could not get global options")?;

    let options = global_options.merge(
        settings
            .get(&format!("options.{}", options_name))
            .context("Could not get provided option set from config")?,
    );

    let mut command = Command::new("./arma3server");

    if let Some(name) = options.config {
        command.arg(&format!(
            "-config={}",
            settings
                .get_str(&format!("config.{}.path", name))
                .context("Could not get config file path from config")?
        ));
    }

    arg!(command, options.basic, "cfg");
    arg!(command, options.port, "port");
    arg!(command, options.ranking, "ranking");
    arg_bool!(
        command,
        options.load_mission_to_memory,
        "loadMissionToMemory"
    );
    arg!(command, options.bandwidth_algorithm, "bandwidthAlg");
    arg!(command, options.cpu_count, "cpuCount");
    arg!(command, options.ex_threads, "exThreads");
    arg_bool!(command, options.enable_ht, "enableHT");
    arg_bool!(command, options.hugepages, "hugepages");

    command.spawn()?;

    Ok(())
}
