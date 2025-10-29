use crate::WINDOW_LIVE_LABEL;
use tauri::Manager;
use window_vibrancy::apply_blur;

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HeaderInfo {
    pub total_dps: f64,
    pub total_dmg: u128,
    pub elapsed_ms: u128,
    pub fight_start_timestamp_ms: u128, // Unix timestamp when fight started
}

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayersWindow {
    pub player_rows: PlayerRows,
}

pub type PlayerRows = Vec<PlayerRow>;

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRow {
    pub uid: u128,
    pub name: String,
    pub class_name: String,
    pub class_spec_name: String,
    pub ability_score: u128,
    pub total_dmg: u128,
    pub dps: f64,
    pub dmg_pct: f64,
    pub crit_rate: f64,
    pub crit_dmg_rate: f64,
    pub lucky_rate: f64,
    pub lucky_dmg_rate: f64,
    pub hits: u128,
    pub hits_per_minute: f64,
    // Extended player attributes from Stage 4
    pub rank_level: Option<i64>,
    pub current_hp: Option<i64>,
    pub max_hp: Option<i64>,
    pub crit_stat: Option<i64>,
    pub lucky_stat: Option<i64>,
    pub haste: Option<i64>,
    pub mastery: Option<i64>,
    pub element_flag: Option<i64>,
    pub energy_flag: Option<i64>,
    pub reduction_level: Option<i64>,
}

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillsWindow {
    pub curr_player: PlayerRows,
    pub skill_rows: SkillRows,
}

pub type SkillRows = Vec<SkillRow>;

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillRow {
    pub name: String,
    pub total_dmg: u128,
    pub dps: f64,
    pub dmg_pct: f64,
    pub crit_rate: f64,
    pub crit_dmg_rate: f64,
    pub lucky_rate: f64,
    pub lucky_dmg_rate: f64,
    pub hits: u128,
    pub hits_per_minute: f64,
}
