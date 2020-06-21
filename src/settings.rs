use anyhow::{Context, Result};
use config::{Config, Environment, File, Value};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Could not merge config")]
    MergeConfig,

    #[error("Could not set default value")]
    SetDefault,

    #[error("Could not get key")]
    Get,

    #[error("Could not get string key")]
    GetStr,

    #[error("Could not get int key")]
    GetInt,

    #[error("Could not get float key")]
    GetFloat,

    #[error("Could not get bool key")]
    GetBool,

    #[error("Could not get table key")]
    GetTable,

    #[error("Could not get array key")]
    GetArray,
}

pub struct Settings(Config);

impl Settings {
    pub fn from_path<P>(path: Option<P>) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut settings = Config::default();

        if let Some(path) = path {
            let file: File<_> = path.as_ref().into();
            settings.merge(file).context(SettingsError::MergeConfig)?
        } else {
            // Add in `./amraam.toml`
            settings
                .merge(File::with_name("amraam"))
                .context(SettingsError::MergeConfig)?
        }
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(Environment::with_prefix("AMRAAM"))
        .context(SettingsError::MergeConfig)?;

        Ok(Self(settings))
    }

    pub fn set_default<T>(&mut self, key: &str, value: T) -> Result<&mut Config>
    where
        T: Into<Value>,
    {
        self.0
            .set_default(key, value)
            .context(SettingsError::SetDefault)
    }

    pub fn get<'de, T: Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        match self.0.get(key) {
            Ok(v) => Ok(Some(v)),
            Err(config::ConfigError::NotFound(_)) => Ok(None),
            Err(err) => Err(err).context(SettingsError::Get),
        }
    }

    pub fn get_str(&self, key: &str) -> Result<Option<String>> {
        match self.0.get_str(key) {
            Ok(v) => Ok(Some(v)),
            Err(config::ConfigError::NotFound(_)) => Ok(None),
            Err(err) => Err(err).context(SettingsError::GetStr),
        }
    }

    pub fn get_int(&self, key: &str) -> Result<Option<i64>> {
        match self.0.get_int(key) {
            Ok(v) => Ok(Some(v)),
            Err(config::ConfigError::NotFound(_)) => Ok(None),
            Err(err) => Err(err).context(SettingsError::GetInt),
        }
    }

    pub fn get_float(&self, key: &str) -> Result<Option<f64>> {
        match self.0.get_float(key) {
            Ok(v) => Ok(Some(v)),
            Err(config::ConfigError::NotFound(_)) => Ok(None),
            Err(err) => Err(err).context(SettingsError::GetFloat),
        }
    }

    pub fn get_bool(&self, key: &str) -> Result<Option<bool>> {
        match self.0.get_bool(key) {
            Ok(v) => Ok(Some(v)),
            Err(config::ConfigError::NotFound(_)) => Ok(None),
            Err(err) => Err(err).context(SettingsError::GetBool),
        }
    }

    pub fn get_table(&self, key: &str) -> Result<Option<HashMap<String, Value>>> {
        match self.0.get_table(key) {
            Ok(v) => Ok(Some(v)),
            Err(config::ConfigError::NotFound(_)) => Ok(None),
            Err(err) => Err(err).context(SettingsError::GetTable),
        }
    }

    pub fn get_array(&self, key: &str) -> Result<Option<Vec<Value>>> {
        match self.0.get_array(key) {
            Ok(v) => Ok(Some(v)),
            Err(config::ConfigError::NotFound(_)) => Ok(None),
            Err(err) => Err(err).context(SettingsError::GetArray),
        }
    }
}
