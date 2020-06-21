use amraam::{
    config::OptionSet,
    modpack::{Modpack, ModpackConfig},
    Settings,
};
use anyhow::{bail, ensure, Context, Result};
use clap::ArgMatches;
use std::convert::TryInto;
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
    let settings =
        Settings::from_path(matches.value_of("config")).context("Could not load settings")?;

    let mut options = OptionSet::new();

    if let Some(globals) = settings
        .get::<OptionSet>("options.global")
        .context("Could not get global options")?
    {
        options.merge(globals)
    }

    if let Some(options_name) = matches.value_of("option set") {
        if let Some(set) = settings
            .get(&format!("options.{}", options_name))
            .context("Could not get provided option set")?
        {
            options.merge(set)
        } else {
            bail!("Could nof find option set in config")
        }
    }

    let server_path = settings
        .get_str("server.path")
        .context("Could not read server path from config")?
        .unwrap_or(String::from("./arma3"));

    // allow changing
    let server_binary = "./arma3serverprofiling_x64";

    let arma_user = settings
        .get_str("server.user")
        .context("Could not get server user")?
        .context("Missing server.user key in config")?;

    let mut command = Command::new("sudo");
    command.current_dir(server_path);
    command.args(&["-u", &arma_user, &server_binary]);

    if let Some(name) = options.config {
        let config = settings
            .get_str(&format!("config.{}.path", name))
            .context("Could not get config file path from config")?
            .unwrap_or_else(|| name);
        command.arg(&format!("-config={}", config));
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

    if let Some(modpack_name) = options.modpack {
        let modpack_config: ModpackConfig = settings
            .get(&format!("modpack.{}", modpack_name))
            .context("Could not get modpack from config")?
            .context("Missing modpack config entry")?;

        let modpack: Modpack = modpack_config.try_into()?;

        command.arg(format!("-mod={}", modpack.as_arg()));
    }

    ensure!(
        command
            .status()
            .context("Could not execute arma3server")?
            .success(),
        "Arma Server did not return sucessfully"
    );

    Ok(())
}
