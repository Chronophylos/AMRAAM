use crate::commands::prelude::*;
use amraam::{
    config::{BasicConfig, Profile, ServerConfig},
    tools::{chmod, chown},
    util::escalate_if_needed,
};
use console::{style, Term};
use dialoguer::{Editor, Input};
use std::{fs::create_dir_all, path::Path};
use thiserror::Error;

macro_rules! input_config {
    ($stdout:ident, $config:expr, $typ:ty, $prompt:expr) => {
        $config = Input::<$typ>::new()
            .with_prompt($prompt)
            .default($config)
            .interact()
            .context(GenerateError::Interact)?;
        $stdout.write_line("").context(GenerateError::WriteLine)?;
    };
}

macro_rules! editor_config {
    ($stdout:ident, $config:expr, $prompt:expr) => {
        $config = Editor::new()
            .trim_newlines(false)
            .edit(&format!("{}:\n{}", $prompt, $config.join("\n")))
            .context(GenerateError::Edit)?
            .map(|m| {
                m.lines()
                    .skip(1) // skip first line
                    .map(|l| l.to_owned())
                    .collect::<Vec<String>>()
            })
            .unwrap_or($config);
        $stdout.write_line("").context(GenerateError::WriteLine)?;
    };
}

#[derive(Error, Debug)]
pub enum GenerateError {
    #[error("Could not change ownership of path")]
    Chown,

    #[error("Could not set permission of path")]
    Chmod,

    #[error("Could not generate config file")]
    GenerateConfig,

    #[error("Could not write line to stdout")]
    WriteLine,

    #[error("Could not interact with user")]
    Interact,

    #[error("Could not open editor")]
    Edit,
}

pub fn cli() -> App {
    SubCommand::with_name("generate")
        .about("Generate config files")
        .args(&[
            Arg::with_name("type")
                .required(true)
                .possible_values(&["basic", "profile", "server"]),
            Arg::with_name("name")
                .required(true)
                .help("Sets the config name")
                .long_help("Sets the config name. A extension must not be provided and is automatically added."),
            Arg::with_name("default").long("default"),
            Arg::with_name("force").short("f").long("force"),
        ])
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    match escalate_if_needed() {
        Ok(_) => {}
        Err(err) => bail!("Could not escalate with sudo: {}", err),
    };

    let settings =
        Settings::from_path(args.value_of("config")).context("Could not load settings")?;

    let name = args.value_of("name").context("Missing argument `name`")?;

    match args.value_of("type").context("Missing argument `type`")? {
        "basic" => generate_basic(settings, name, args),
        "profile" => generate_profile(settings, name, args),
        "server" => generate_server(settings, name, args),
        _ => bail!("Unknown type"),
    }
}

fn generate_basic(settings: Settings, name: &str, args: &ArgMatches) -> Result<()> {
    let stdout = Term::stdout();
    let server_path = settings
        .get_server_path()
        .context("Could not get server path from config")?;

    let config_path = Path::new(&server_path).join(&format!("{}.cfg", name));

    if config_path.exists() && !args.is_present("force") {
        bail!("Config already exists. If you want to overwrite this file pass --force");
    }

    let user = settings
        .get_str("server.user")
        .context("Could not get server user from config")?
        .context("Missing config key server.user")?;

    let mut config = BasicConfig::default();

    if !args.is_present("default") {
        input_config!(stdout, config.max_msg_send, u16,
"Maximum number of packets (aggregate messages) that can be sent in one simulation cycle (`frame`).
Increasing this value can decrease lag on high upload bandwidth servers");

        input_config!(stdout, config.max_size_guaranteed, u16,
"Maximum size (payload) of guaranteed packet in bytes (without headers). Small messages are packed
to larger packets (aggregate messages). Guaranteed packets (aggregate messages) are used for
non-repetitive events like shooting. A value over 1300 can cause negative effects"
            );

        input_config!(stdout, config.max_size_nonguaranteed, u16,
"Maximum size (payload) of non-guaranteed packet in bytes (without headers). Small messages are
packed to larger packets (aggregate messages). Non-guaranteed packets (aggregate messages) are used
for repetitive updates like soldier or vehicle position. Increasing this value may improve
bandwidth requirement, but it may increase lag. A value over 1300 can cause negative effects"
            );

        input_config!(stdout, config.min_bandwidth, u32,
"Bandwidth the server is guaranteed to have (in bps). This value helps server to estimate bandwidth
available. Increasing it to too optimistic values can increase lag and CPU load, as too many
messages will be sent but discarded"
            );

        input_config!(stdout, config.max_bandwidth, u32,
"Bandwidth the server is guaranteed to never have (in bps). This value helps the server to estimate
bandwidth available"
            );

        input_config!(stdout, config.min_error_to_send, String,
"Minimal error to send updates across network. Using a smaller value can make units  observed by
binoculars or sniper rifle to move smoother at the trade off of increased network traffic"
                );

        input_config!(stdout, config.min_error_to_send_near, String,
"Minimal error to send updates across network for near units. Using larger value can reduce traffic
sent for near units. Used to control client to server traffic as well");

        input_config!(stdout, config.max_custom_file_size, u32,
"Users with custom face or custom sound larger than this size are kicked when trying to connect. A
value of 0 means no restrictions");
    }

    config
        .generate(&config_path)
        .context(GenerateError::GenerateConfig)?;

    chown(&config_path, &user, false).context(GenerateError::Chown)?;
    chmod(&config_path, 0o755, 0o644, false).context(GenerateError::Chmod)?;

    stdout
        .write_line(&format!(
            " {} generated new basic config",
            style("Successfully").green().bold()
        ))
        .context(GenerateError::WriteLine)?;

    Ok(())
}

// TODO: place config in ~/.local/share/Arma 3 - Other Profiles/name/name.arma3profile
fn generate_profile(settings: Settings, name: &str, args: &ArgMatches) -> Result<()> {
    let stdout = Term::stdout();

    let config_dir_path = Path::new(&dirs::data_dir().context("Could not find data dir")?)
        .join("Arma 3 - Other Profiles")
        .join(&name);
    let config_path = config_dir_path.join(&format!("{}.arma3profile", &name));

    if config_path.exists() && !args.is_present("force") {
        bail!("Config already exists. If you want to overwrite this file pass --force");
    }

    let user = settings
        .get_str("server.user")
        .context("Could not get server user from config")?
        .context("Missing config key server.user")?;

    if !config_dir_path.exists() {
        stdout
            .write_line(&format!(
                " {} config directory",
                style("Creating").blue().bold()
            ))
            .context(GenerateError::WriteLine)?;

        create_dir_all(&config_dir_path).context("Could not create config_dir directory")?;
        chown(&config_dir_path, &user, true).context(GenerateError::Chown)?;
        chmod(&config_dir_path, 0o755, 0o644, true).context(GenerateError::Chmod)?;

        stdout.write_line("").context(GenerateError::WriteLine)?;
    }

    let mut config = Profile::default();

    if !args.is_present("default") {}

    config
        .generate(&config_path)
        .context(GenerateError::GenerateConfig)?;

    chown(&config_path, &user, false).context(GenerateError::Chown)?;
    chmod(&config_path, 0o755, 0o644, false).context(GenerateError::Chmod)?;

    stdout
        .write_line(&format!(
            " {} generated new server config",
            style("Successfully").green().bold()
        ))
        .context(GenerateError::WriteLine)?;

    Ok(())
}

fn generate_server(settings: Settings, name: &str, args: &ArgMatches) -> Result<()> {
    let stdout = Term::stdout();
    let server_path = settings
        .get_server_path()
        .context("Could not get server path from config")?;

    let config_path = Path::new(&server_path).join(&format!("{}.cfg", name));

    if config_path.exists() && !args.is_present("force") {
        bail!("Config already exists. If you want to overwrite this file pass --force");
    }

    let user = settings
        .get_str("server.user")
        .context("Could not get server user from config")?
        .context("Missing config key server.user")?;

    let mut config = ServerConfig::default();

    if !args.is_present("default") {
        input_config!(
            stdout,
            config.hostname,
            String,
            "Servername visible in the game browser"
        );

        input_config!(
            stdout,
            config.password,
            String,
            "Password required to connect to server"
        );

        input_config!(
            stdout,
            config.password_admin,
            String,
            "Password to protect admin access"
        );

        editor_config!(stdout, config.motd, "Enter Message of the Day");

        input_config!(
            stdout,
            config.motd_interval,
            u16,
            "Time interval (in seconds) between each message"
        );

        editor_config!(
            stdout,
            config.admins,
            "Enter admin steamd user ids. Whitelisted clients can use #login w/o password"
        );

        input_config!(
            stdout,
            config.max_players,
            u8,
            "The maximum number of players that can connect to server. The
final number will be lesser between number given here and number
of mission slots (default value is 64 for dedicated server)."
        );

        input_config!(
            stdout,
            config.steam_protocol_max_data_size,
            u16,
            "Limit for maximum Steam Query packet length. (since Arma 3 1.99+) Increasing
this value is dangerous as it can cause Arma server to send UDP packets of a
size larger than the MTU. This will cause UDP packets to be fragmented which
is not supported by some older routers. But increasing this will fix the
modlist length limit in Arma 3 Launcher"
        );
    }

    config
        .generate(&config_path)
        .context(GenerateError::GenerateConfig)?;

    chown(&config_path, &user, false).context(GenerateError::Chown)?;
    chmod(&config_path, 0o755, 0o644, false).context(GenerateError::Chmod)?;

    stdout
        .write_line(&format!(
            " {} generated new server config",
            style("Successfully").green().bold()
        ))
        .context(GenerateError::WriteLine)?;

    Ok(())
}
