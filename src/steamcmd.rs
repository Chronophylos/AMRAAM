use anyhow::{bail, ensure, Context, Result};
use std::process::Command;

static ARMA_SERVER_APPID: &str = "233780";
static ARMA_APPID: &str = "107410";

#[derive(Default)]
pub struct SteamCmd<'a> {
    sudo: &'a str,
    username: &'a str,
    password: Option<&'a str>,
    token: Option<&'a str>,
    install_dir: &'a str,
}

impl<'a> SteamCmd<'a> {
    pub fn new(
        sudo: &'a str,
        username: &'a str,
        password: &'a str,
        token: &'a str,
        install_dir: &'a str,
    ) -> Self {
        Self {
            sudo,
            username,
            password: Some(password),
            token: Some(token),
            install_dir,
        }
    }

    fn run(&self, args: &[&'a str]) -> Result<()> {
        let status = Command::new("sudo")
            .args(&[
                "-iu",
                self.sudo,
                SteamCmd::binary_path()?,
                "+login",
                self.username,
                self.password.unwrap_or(""),
                self.password.and(self.token).unwrap_or(""),
                "+force_install_dir",
                self.install_dir,
            ])
            .args(args)
            .arg("+quit")
            .status()
            .context("Could not run SteamCMD binary (maybe sudo is missing)")?;

        ensure!(status.success(), "SteamCMD did not complete successfully");

        Ok(())
    }

    pub fn update_mod(&self, mod_id: &str) -> Result<()> {
        self.run(&["+workshop_download_item", ARMA_APPID, mod_id])
    }

    pub fn update_mods(&self, mod_ids: &[&str]) -> Result<()> {
        self.run(&[&["+workshop_download_item", ARMA_APPID], mod_ids].concat())
    }

    pub fn update_arma(&self) -> Result<()> {
        self.run(&[
            "+app_update",
            ARMA_SERVER_APPID,
            "-validate",
            "-beta",
            "profiling",
            "-betapassword",
            "CautionSpecialProfilingAndTestingBranchArma3",
        ])
    }

    fn binary_path<'p>() -> Result<&'p str> {
        use os_info::Type::*;
        let info = os_info::get();

        match info.os_type() {
            Debian | Ubuntu => Ok("/usr/games/steamcmd"),
            _ => bail!("Cannot install steamcmd for this operating system."),
        }
    }
}
