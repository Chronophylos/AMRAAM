use anyhow::Result;
use chrono::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tinytemplate::TinyTemplate;

#[derive(Serialize)]
pub struct BasicConfig {
    pub max_msg_send: u16,
    pub max_size_guaranteed: u16,
    pub max_size_nonguaranteed: u16,
    pub min_bandwidth: u32,
    pub max_bandwidth: u32,
    // floats are annoying to format
    pub min_error_to_send: String,
    pub min_error_to_send_near: String,
    pub max_custom_file_size: u32,
}

impl Default for BasicConfig {
    fn default() -> Self {
        Self {
            max_msg_send: 128,
            max_size_guaranteed: 512,
            max_size_nonguaranteed: 256,
            min_bandwidth: 131072,
            max_bandwidth: 125000000,
            min_error_to_send: "0.001".into(),
            min_error_to_send_near: "0.01".into(),
            max_custom_file_size: 0,
        }
    }
}

#[derive(Serialize)]
struct Context<'a> {
    timestamp: String,
    config: &'a BasicConfig,
}

impl BasicConfig {
    pub fn generate<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        lazy_static! {
            static ref TEMPLATE: &'static str = include_str!("../../assets/basic.cfg.in");
        }

        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&super::format);
        tt.add_template("template", &TEMPLATE)?;

        let context = Context {
            timestamp: Local::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            config: self,
        };

        let rendered = tt.render("template", &context)?;

        let mut file = File::create(path)?;
        write!(file, "{}", rendered)?;

        Ok(())
    }
}
