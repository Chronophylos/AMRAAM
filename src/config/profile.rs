use anyhow::Result;
use chrono::prelude::*;
use serde_repr::Serialize_repr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tinytemplate::TinyTemplate;

#[derive(Serialize_repr)]
#[repr(u8)]
pub enum DistanceOption {
    Never,
    LimitedDistance,
    Always,
}

#[derive(Serialize_repr)]
#[repr(u8)]
pub enum FadeOption {
    Never,
    FadeOut,
    Always,
}

#[derive(Serialize_repr)]
#[repr(u8)]
pub enum AiLevelPreset {
    Low,
    Normal,
    High,
    Custom,
}

#[derive(Serialize)]
pub struct Profile<'a> {
    reduced_damage: bool,
    group_indicators: DistanceOption,
    friendly_tags: DistanceOption,
    enemy_tags: DistanceOption,
    detected_mines: DistanceOption,
    commands: FadeOption,
    waypoints: FadeOption,
    tactical_ping: bool,
    weapon_info: FadeOption,
    stance_indicator: FadeOption,
    stamina_bar: bool,
    weapon_crosshair: bool,
    vision_aid: bool,
    third_person_view: bool,
    camera_shake: bool,
    score_table: bool,
    death_messages: bool,
    von_id: bool,
    map_content_friendly: bool,
    map_content_enemies: bool,
    map_content_mines: bool,
    auto_report: bool,
    multiple_saves: bool,
    ai_level_preset: AiLevelPreset,
    skill_ai: &'a str,
    precision_ai: &'a str,
}

impl Default for Profile<'_> {
    fn default() -> Self {
        Self {
            reduced_damage: false,
            group_indicators: DistanceOption::LimitedDistance,
            friendly_tags: DistanceOption::Never,
            enemy_tags: DistanceOption::Never,
            detected_mines: DistanceOption::Never,
            commands: FadeOption::FadeOut,
            waypoints: FadeOption::FadeOut,
            tactical_ping: false,
            weapon_info: FadeOption::FadeOut,
            stance_indicator: FadeOption::FadeOut,
            stamina_bar: true,
            weapon_crosshair: false,
            vision_aid: false,
            third_person_view: false,
            camera_shake: true,
            score_table: true,
            death_messages: false,
            von_id: false,
            map_content_friendly: true,
            map_content_enemies: false,
            map_content_mines: false,
            auto_report: false,
            multiple_saves: false,
            ai_level_preset: AiLevelPreset::Custom,
            skill_ai: "0.7",
            precision_ai: "0.4",
        }
    }
}

#[derive(Serialize)]
struct Context<'a> {
    timestamp: String,
    config: &'a Profile<'a>,
}

impl Profile<'_> {
    pub fn generate<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        lazy_static! {
            static ref TEMPLATE: &'static str = include_str!("../../assets/server.armaprofile.in");
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
