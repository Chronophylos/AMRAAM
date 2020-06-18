//! TODO: do mission, voting and scripting related config entires

use anyhow::Result;
use chrono::prelude::*;
use serde::Serializer;
use serde_repr::Serialize_repr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tinytemplate::TinyTemplate;

#[derive(Serialize_repr)]
#[repr(u8)]
pub enum SignatureVerification {
    Disabled,
    V1orV2,
    V2Only,
}

#[derive(Serialize_repr)]
#[repr(u8)]
pub enum FilePatching {
    Disallow,
    AllowHeadless,
    AllowAll,
}

#[derive(Serialize_repr)]
#[repr(u8)]
pub enum VonCodec {
    OPUS,
    SPEEX,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TimestampFormat {
    None,
    Short,
    Full,
}

#[derive(Serialize)]
pub struct KickClientsOnSlowNetwork {
    pub max_ping: bool,
    pub max_packet_loss: bool,
    pub max_desync: bool,
    pub disconnect_timeout: bool,
}

#[derive(Serialize_repr)]
#[repr(u8)]
pub enum KickType {
    Manual,
    Connectivity,
    BattlEye,
    Harmless,
}

pub enum KickTimeout {
    ServerRestart,
    MissionEnd,
    Second(u16),
}

impl serde::Serialize for KickTimeout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::ServerRestart => serializer.serialize_i8(-2),
            Self::MissionEnd => serializer.serialize_i8(-1),
            Self::Second(n) => serializer.serialize_u16(*n),
        }
    }
}

#[derive(Serialize)]
pub struct KickDefinition {
    #[serde(rename = "type")]
    pub typ: KickType,
    pub timeout: KickTimeout,
}

#[derive(Serialize)]
pub struct TimeoutDefinition {
    pub ready: u16,
    pub not_ready: u16,
}

#[derive(Serialize_repr)]
#[repr(u8)]
pub enum RotorLibSimulation {
    Ignore,
    Advanced,
    Simple,
}

#[derive(Serialize)]
pub enum Difficulty {
    Recruit,
    Regular,
    Veteran,
    Custom,
}

#[derive(Serialize)]
pub struct ServerConfig<'a> {
    pub hostname: &'a str,
    pub password: &'a str,
    pub password_admin: &'a str,
    pub log_file: &'a str,
    pub motd: &'a [&'a str],
    pub motd_interval: u16,
    pub admins: &'a [&'a str],
    pub steam_protocol_max_data_size: u16,
    pub max_players: u8,
    pub kick_duplicate: bool,
    pub verify_signatures: SignatureVerification,
    pub allowed_file_patching: FilePatching,
    pub file_patching_exceptions: &'a [&'a str],
    pub required_build: &'a str,
    pub vote_mission_players: u8,
    pub vote_threshold: &'a str,
    pub disable_von: bool,
    pub von_codec: VonCodec,
    pub von_codec_quality: u8,
    pub persistent: bool,
    pub timestamp_format: TimestampFormat,
    pub battleye: bool,
    pub allowed_load_file_extensions: &'a [&'a str],
    pub allowed_preprocess_file_extensions: &'a [&'a str],
    pub allowed_html_load_extensions: &'a [&'a str],
    pub disconnect_timeout: u16,
    pub max_desync: u16,
    pub max_ping: u16,
    pub max_packet_loss: u16,
    pub kick_clients_on_slow_network: KickClientsOnSlowNetwork,
    pub kick_timeout: &'a [KickDefinition],
    pub voting_timeout: TimeoutDefinition,
    pub role_timeout: TimeoutDefinition,
    pub briefing_timeout: TimeoutDefinition,
    pub debriefing_timeout: TimeoutDefinition,
    pub lobby_idle_timeout: u16,
    pub force_rotor_lib_simulation: RotorLibSimulation,
    pub statistics_enabled: bool,
    pub forced_difficulty: Difficulty,
    pub mission_whitelist: &'a [&'a str],
}

impl Default for ServerConfig<'_> {
    fn default() -> Self {
        Self {
            hostname: "Fun and Test Server",
            password: "",
            password_admin: "xyz",
            log_file: "server_console.log",
            motd: &[
                "",
                "",
                "Two empty lines above for increasing interval",
                "Welcome to our server",
                "",
                "",
                "We are looking for fun - Join us Now !",
                "http://www.example.com",
                "One more empty line below for increasing interval",
                "",
            ],
            motd_interval: 5,
            admins: &[],
            steam_protocol_max_data_size: 1024,
            max_players: 16,
            kick_duplicate: true,
            verify_signatures: SignatureVerification::V2Only,
            allowed_file_patching: FilePatching::Disallow,
            file_patching_exceptions: &[],
            required_build: "0",
            vote_mission_players: 1,
            vote_threshold: "0.33",
            disable_von: false,
            von_codec: VonCodec::OPUS,
            von_codec_quality: 30,
            persistent: true,
            timestamp_format: TimestampFormat::Short,
            battleye: true,
            allowed_load_file_extensions: &[
                "hpp", "sqs", "sqf", "fsm", "cpp", "paa", "txt", "xml", "inc", "ext", "sqm", "ods",
                "fxy", "lip", "csv", "kb", "bik", "bikb", "html", "htm", "biedi",
            ],
            allowed_preprocess_file_extensions: &[
                "hpp", "sqs", "sqf", "fsm", "cpp", "paa", "txt", "xml", "inc", "ext", "sqm", "ods",
                "fxy", "lip", "csv", "kb", "bik", "bikb", "html", "htm", "biedi",
            ],
            allowed_html_load_extensions: &["htm", "html", "xml", "txt"],
            disconnect_timeout: 5,
            max_desync: 150,
            max_ping: 200,
            max_packet_loss: 50,
            kick_clients_on_slow_network: KickClientsOnSlowNetwork {
                max_ping: false,
                max_packet_loss: false,
                max_desync: false,
                disconnect_timeout: false,
            },
            kick_timeout: &[
                KickDefinition {
                    typ: KickType::Manual,
                    timeout: KickTimeout::MissionEnd,
                },
                KickDefinition {
                    typ: KickType::Connectivity,
                    timeout: KickTimeout::Second(180),
                },
                KickDefinition {
                    typ: KickType::BattlEye,
                    timeout: KickTimeout::Second(180),
                },
                KickDefinition {
                    typ: KickType::Harmless,
                    timeout: KickTimeout::Second(180),
                },
            ],
            voting_timeout: TimeoutDefinition {
                ready: 60,
                not_ready: 90,
            },
            role_timeout: TimeoutDefinition {
                ready: 60,
                not_ready: 120,
            },
            briefing_timeout: TimeoutDefinition {
                ready: 60,
                not_ready: 90,
            },
            debriefing_timeout: TimeoutDefinition {
                ready: 45,
                not_ready: 60,
            },
            lobby_idle_timeout: 300,
            force_rotor_lib_simulation: RotorLibSimulation::Ignore,
            statistics_enabled: true,
            forced_difficulty: Difficulty::Regular,
            mission_whitelist: &[],
        }
    }
}

#[derive(Serialize)]
struct Context<'a> {
    timestamp: String,
    config: &'a ServerConfig<'a>,
}

impl ServerConfig<'_> {
    pub fn generate<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        lazy_static! {
            static ref TEMPLATE: &'static str = include_str!("../../assets/server.cfg.in");
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
