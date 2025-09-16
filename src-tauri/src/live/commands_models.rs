#[derive(specta::Type)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DPSWindow {
    pub damage_rows: DPSRows,
    pub total_damage: u128,
}

pub type DPSRows = Vec<DPSRow>;

#[derive(specta::Type)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DPSRow {
    pub uid: i64,
    pub name: String,
    pub class: String,
    // pub class_style: String,
    pub ability_score: i32,
    pub total_damage: u128,
    pub dps: f64,
}
