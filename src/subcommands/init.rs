use amraam::steamcmd::SteamCmd;
use anyhow::{bail, ensure, Context, Result};
use clap::ArgMatches;
use console::{Style, Term};
use dialoguer::{Input, Password};
use lazy_static::lazy_static;
use pwd::Passwd;
use std::{
    fs::File,
    io::prelude::*,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::Path,
    process::Command,
};
use thiserror::Error;
use toml::map::Map;

static STEPS: &'static str = "5";
lazy_static! {
    static ref STEP_STYLE: Style = Style::new().bold();
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Could not interact with input prompt")]
    InputInteract,

    #[error("Could not interact with password prompt")]
    PasswordInteract,
}

pub fn init(_matches: &ArgMatches) -> Result<()> {
    ensure!(
        console::user_attended(),
        "This command is interactively and shouldn't be used in a script"
    );

    check_os()?;

    match sudo::escalate_if_needed() {
        Ok(_) => {}
        Err(err) => bail!("Could not escalate with sudo: {}", err),
    };

    let mut settings = Map::new();
    let term = Term::stdout();

    // create user if needed
    term.write_line(&format!("[1/{}] Setup server user", STEPS))?;
    let name = Input::<String>::new()
        .with_prompt("Server username")
        .default("arma".into())
        .interact()
        .context(InitError::InputInteract)?;

    let user = match Passwd::from_name(&name) {
        Ok(u) => u,
        Err(err) => bail!("Could not get user from pwd: {}", err),
    };

    let target_dir;
    let user = match user {
        Some(user) => {
            info!("Found existing user");
            term.write_line(
                "I found an exising user with the same name. We'll be using this user",
            )?;

            target_dir = Input::<String>::new()
                .with_prompt("Server installation directory")
                .default(user.dir.clone())
                .interact()
                .context(InitError::InputInteract)?;

            user
        }
        None => {
            create_user(&name).context("Could not create user")?;

            let user = match Passwd::from_name(&name) {
                Ok(u) => u,
                Err(err) => bail!("Could not get user from pwd: {}", err),
            }
            .context("User was not successfully created")?;

            target_dir = user.dir.to_owned();

            user
        }
    };

    let uid = user.uid;
    let target_path = Path::new(&target_dir);

    settings.insert("server.uid".into(), uid.into());

    // install steamcmd
    term.write_line(&format!("\n[2/{}] Installing SteamCMD", STEPS))?;
    install_steamcmd().context("Could not install SteamCMD")?;

    // install arma server
    term.write_line(&format!("\n[3/{}] Installing Arma 3 via SteamCMD", STEPS))?;

    let server_path = target_path.join("arma3");
    install_server(server_path, &user.name)?;

    // save config
    term.write_line(&format!("[4/{}] Saving config", STEPS))?;
    let config_file = target_path.join("amraam.toml");
    let serialized = toml::to_string_pretty(&settings).context("Could not serialize config")?;

    let mut file = File::create(&config_file)?;
    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o600);
    file.set_permissions(perms)?;
    write!(file, "{}", serialized)?;
    file.sync_all()?;

    // set ownership of all files
    term.write_line(&format!("[5/{}] Setting file permissions", STEPS))?;
    chown(&config_file, &name, &name)?;

    Ok(())
}

fn check_os() -> Result<()> {
    use os_info::Type::*;
    let info = os_info::get();

    match info.os_type() {
        Debian | Ubuntu => Ok(()),
        os => bail!(
            "Cannot init AMRAAM on {}. If you think this should change please open a pull request.",
            os
        ),
    }
}

fn chown<P>(path: P, user: &str, group: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    let status = Command::new("chown")
        .args(&[
            "--recursive",
            &format!("{}:{}", user, group),
            path.as_ref().to_str().unwrap(),
        ])
        .status()
        .context("Could not execute chown")?;

    ensure!(status.success(), "chown did not return successfully");

    Ok(())
}

fn create_user(name: &str) -> Result<()> {
    let password = Password::new()
        .with_prompt(format!("Enter password for new user {}", name))
        .with_confirmation("Confirm password", "Passwords mismatching")
        .interact()
        .context(InitError::PasswordInteract)?;

    let home_dir = Input::<String>::new()
        .with_prompt("Server installation directory")
        .default(format!("/home/{}", name))
        .interact()
        .context(InitError::InputInteract)?;

    let status = Command::new("useradd")
        .args(&[
            "--create-home",
            "--system",
            "--user-group",
            "--home-dir",
            &home_dir,
            //"--shell",
            //"/usr/bin/bash",
            "--password",
            &password,
            name,
        ])
        .uid(0) // run as root
        .status()
        .context("Could not execute useradd with sudo")?;

    ensure!(status.success(), "sudo useradd did not return successfully");

    info!("Created server user `{}`", name);

    Ok(())
}

fn install_steamcmd() -> Result<()> {
    use os_info::Type::*;
    let info = os_info::get();

    info!("Installing steamcmd");

    match info.os_type() {
        Debian => install_steamcmd_debian(info.bitness(), true)
            .context("Could not install SteamCMD with Debian Strategy"),
        Ubuntu => install_steamcmd_debian(info.bitness(), false)
            .context("Could not install SteamCMD with Ubuntu/Debian Strategy"),
        _ => bail!("Cannot install steamcmd for this operating system."),
    }
}

static DEBIAN_PACKAGES: &[&'static str] = &["sudo", "steamcmd"];
static DEBIAN_REPOSITORY: &str = "non-free";
static UBUNTU_REPOSITORY: &str = "multiverse";

fn install_steamcmd_debian(bitness: os_info::Bitness, debian: bool) -> Result<()> {
    use os_info::Bitness;
    let repository = if debian {
        DEBIAN_REPOSITORY
    } else {
        UBUNTU_REPOSITORY
    };

    match bitness {
        Bitness::X64 => {
            ensure!(
                Command::new("add-apt-repository")
                    .args(&["--yes", repository])
                    .status()
                    .context("Could not run add-apt-repository")?
                    .success(),
                "Could not add multiverse repository"
            );
            ensure!(
                Command::new("dpkg")
                    .args(&["--add-architecture", "i386"])
                    .status()
                    .context("Could not run dpkg")?
                    .success(),
                "Could not add 32 bit architecture"
            );
            ensure!(
                Command::new("apt")
                    .args(&["--assume-yes", "update"])
                    .status()
                    .context("Could not run apt")?
                    .success(),
                "Could not update repositories"
            );
            ensure!(
                Command::new("apt")
                    .args([&["--assume-yes", "install", "lib32gcc1"], DEBIAN_PACKAGES].concat())
                    .status()
                    .context("Could not run apt")?
                    .success(),
                "Could not install packages"
            );
        }
        Bitness::X32 => {
            ensure!(
                Command::new("apt")
                    .args(&["--assume-yes", "update"])
                    .status()
                    .context("Could not run apt")?
                    .success(),
                "Could not update repositories"
            );
            ensure!(
                Command::new("apt")
                    .args(&["--assume-yes", "install"])
                    .args(DEBIAN_PACKAGES)
                    .status()
                    .context("Could not run apt")?
                    .success(),
                "Could not install packages"
            );
        }
        _ => bail!("Could not determine bitness of host system"),
    };

    Ok(())
}

fn install_server<P>(path: P, user: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    let username = Input::<String>::new()
        .with_prompt("Steam username")
        .interact()
        .context(InitError::InputInteract)?;

    let password = Password::new()
        .with_prompt(&format!("Enter password for {}", username))
        .with_confirmation("Confirm password", "Passwords mismatching")
        .interact()
        .context(InitError::PasswordInteract)?;

    let token = Input::<String>::new()
        .with_prompt("Enter your 2FA Token")
        .interact()
        .context(InitError::InputInteract)?;

    let steamcmd = SteamCmd::new(
        user,
        &username,
        &password,
        &token,
        path.as_ref().to_str().unwrap(),
    );

    steamcmd
        .update_arma()
        .context("Could not install ArmA 3 Server")?;

    Ok(())
}
