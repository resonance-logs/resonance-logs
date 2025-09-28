#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeaderInfo {
    pub total_dps: f64,
    pub total_dmg: u128,
    pub elapsed_ms: u128,
}

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DPSWindow {
    pub dps_rows: DPSRows,
}

pub type DPSRows = Vec<DPSRow>;

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DPSRow {
    pub uid: i64,
    pub name: String,
    pub class: String,
    pub class_spec: String,
    pub ability_score: i32,
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

#[derive(specta::Type, serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DPSSkillBreakdownWindow {
    pub curr_player: DPSRow,
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
