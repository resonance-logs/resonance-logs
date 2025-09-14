use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use blueprotobuf_lib::blueprotobuf::EEntityType;

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncounterInner {
    pub time_last_combat_packet: i64, // in ms? todo:
    pub time_fight_start: i64, // in ms? todo:
    // pub local_player: i64, // todo: get from SyncToMeDeltaInfo
    pub entities: HashMap<i64, Entity>, // k: entity uuid
    // pub current_boss: Option<Entity>, // todo: is there a way to identify the boss?
    // pub duration: i64,
    // pub reset: bool
}

pub type Encounter = Mutex<EncounterInner>;

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    // pub time_last_update: i64,
    // pub npc_id: i32,
    pub uid: i64,
    pub name: String,
    pub entity_type: EEntityType,
    // pub class_id: i32,
    // pub class: String,
    // pub gear_score: f64,
    // pub current_hp: i64,
    // pub max_hp: i64,
    // pub is_dead: bool,
    // pub skills: HashMap<String, Skill>,
    pub damage_stats: DamageStats,
    // pub skill_stats: SkillStats,
}

// #[derive(Debug, Default, Serialize, PartialEq, Clone)]
// pub enum EntityType {
//     #[default] UNKNOWN,
//     MONSTER,
//     // BOSS,
//     // GUARDIAN,
//     PLAYER,
//     // NPC
// }

// #[derive(Debug, Default, Serialize, Clone)]
// #[serde(rename_all = "camelCase")]
// pub struct Skill {
//     pub id: i32,
//     pub name: String,
//     pub total_damage: i64,
//     pub max_damage: i64,
//     pub casts: i64,
//     pub hits: i64,
//     pub crits: i64,
//     pub back_attacks: i64,
//     pub front_attacks: i64,
//     pub dps: i64,
//     pub dps_intervals: HashMap<i32, i64>,
// }

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DamageStats {
    pub damage_dealt: i64,
    // pub damage_taken: i64,
    // pub deaths: i64,
    // pub death_time: i64,
    // pub dps: i64,
    // pub dps_intervals: HashMap<i32, i64>,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillStats {
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
}
