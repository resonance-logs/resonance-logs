pub type DamageRows = Vec<DamageRow>;

#[derive(specta::Type)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DamageRow {
    pub uid: i64,
    pub name: String,
    pub class: String,
    // pub class_style: String,
    pub ability_score: i32,
    pub total_damage: u128,
    pub dps: f64,
}
