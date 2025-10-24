use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema as sch;

#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = sch::sessions)]
pub struct Session {
    pub id: i32,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub version: Option<String>,
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::sessions)]
pub struct NewSession {
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub version: Option<String>,
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = sch::entities, primary_key(entity_id))]
pub struct EntityRow {
    pub entity_id: i64,
    pub entity_type: i32,
    pub is_player: i32,
    pub name: Option<String>,
    pub class_id: Option<i32>,
    pub class_spec: Option<i32>,
    pub ability_score: Option<i32>,
    pub level: Option<i32>,
    pub first_seen_ms: Option<i64>,
    pub last_seen_ms: Option<i64>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::entities)]
pub struct NewEntity<'a> {
    pub entity_id: i64,
    pub entity_type: i32,
    pub is_player: i32,
    pub name: Option<&'a str>,
    pub class_id: Option<i32>,
    pub class_spec: Option<i32>,
    pub ability_score: Option<i32>,
    pub level: Option<i32>,
    pub first_seen_ms: Option<i64>,
    pub last_seen_ms: Option<i64>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = sch::entities)]
pub struct UpdateEntity<'a> {
    pub entity_type: Option<i32>,
    pub is_player: Option<i32>,
    pub name: Option<&'a str>,
    pub class_id: Option<i32>,
    pub class_spec: Option<i32>,
    pub ability_score: Option<i32>,
    pub level: Option<i32>,
    pub last_seen_ms: Option<i64>,
}

#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = sch::skills, primary_key(skill_id))]
pub struct SkillRow {
    pub skill_id: i32,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::skills)]
pub struct NewSkill<'a> {
    pub skill_id: i32,
    pub name: Option<&'a str>,
}

#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = sch::encounters)]
pub struct EncounterRow {
    pub id: i32,
    pub session_id: Option<i32>,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub local_player_id: Option<i64>,
    pub total_dmg: Option<i64>,
    pub total_heal: Option<i64>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::encounters)]
pub struct NewEncounter {
    pub session_id: Option<i32>,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub local_player_id: Option<i64>,
    pub total_dmg: Option<i64>,
    pub total_heal: Option<i64>,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = sch::damage_events, belongs_to(EncounterRow, foreign_key = encounter_id))]
pub struct DamageEventRow {
    pub id: i32,
    pub encounter_id: i32,
    pub timestamp_ms: i64,
    pub attacker_id: i64,
    pub defender_id: Option<i64>,
    pub skill_id: Option<i32>,
    pub value: i64,
    pub is_crit: i32,
    pub is_lucky: i32,
    pub hp_loss: i64,
    pub shield_loss: i64,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::damage_events)]
pub struct NewDamageEvent {
    pub encounter_id: i32,
    pub timestamp_ms: i64,
    pub attacker_id: i64,
    pub defender_id: Option<i64>,
    pub skill_id: Option<i32>,
    pub value: i64,
    pub is_crit: i32,
    pub is_lucky: i32,
    pub hp_loss: i64,
    pub shield_loss: i64,
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = sch::heal_events, belongs_to(EncounterRow, foreign_key = encounter_id))]
pub struct HealEventRow {
    pub id: i32,
    pub encounter_id: i32,
    pub timestamp_ms: i64,
    pub healer_id: i64,
    pub target_id: Option<i64>,
    pub skill_id: Option<i32>,
    pub value: i64,
    pub is_crit: i32,
    pub is_lucky: i32,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::heal_events)]
pub struct NewHealEvent {
    pub encounter_id: i32,
    pub timestamp_ms: i64,
    pub healer_id: i64,
    pub target_id: Option<i64>,
    pub skill_id: Option<i32>,
    pub value: i64,
    pub is_crit: i32,
    pub is_lucky: i32,
}
