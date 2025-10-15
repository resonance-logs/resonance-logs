use crate::live::commands_models::{HeaderInfo, PlayerRow, SkillRow};
use crate::live::opcodes_models::{Encounter, Entity, Skill, class};
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::{info, trace, warn, error};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveEventType {
    PlayerUpdate,
    SkillUpdate,
    EncounterReset,
    EncounterPause,
    EncounterResume,
    NewDamage,
    NewHeal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveEvent {
    pub event_type: LiveEventType,
    pub data: LiveEventData,
    pub timestamp: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiveEventData {
    PlayerUpdate(PlayerUpdateData),
    SkillUpdate(SkillUpdateData),
    EncounterUpdate(EncounterUpdateData),
    DamageUpdate(DamageUpdateData),
    HealUpdate(HealUpdateData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerUpdateData {
    pub player_uid: i64,
    pub player_row: PlayerRow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillUpdateData {
    pub player_uid: i64,
    pub skill_rows: Vec<SkillRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterUpdateData {
    pub header_info: HeaderInfo,
    pub is_paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DamageUpdateData {
    pub player_uid: i64,
    pub skill_uid: i32,
    pub damage_amount: u128,
    pub is_crit: bool,
    pub is_lucky: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealUpdateData {
    pub player_uid: i64,
    pub skill_uid: i32,
    pub heal_amount: u128,
    pub is_crit: bool,
    pub is_lucky: bool,
}

pub struct EventManager {
    app_handle: Option<AppHandle>,
}

impl EventManager {
    pub fn new() -> Self {
        Self { app_handle: None }
    }

    pub fn initialize(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
        info!("Event manager initialized");
    }

    pub fn emit_event(&self, event: LiveEvent) {
        if let Some(app_handle) = &self.app_handle {
            let event_name = "live-event".to_string();
            let event_type = event.event_type.clone();

            match app_handle.emit(&event_name, event) {
                Ok(_) => {
                    trace!("Emitted live event: {:?}", event_type);
                }
                Err(e) => {
                    eprintln!("Failed to emit live event: {}", e);
                }
            }
        }
    }

    pub fn emit_player_update(&self, player_uid: i64, player_row: PlayerRow) {
        let event = LiveEvent {
            event_type: LiveEventType::PlayerUpdate,
            data: LiveEventData::PlayerUpdate(PlayerUpdateData {
                player_uid,
                player_row,
            }),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        };
        self.emit_event(event);
    }

    pub fn emit_skill_update(&self, player_uid: i64, skill_rows: Vec<SkillRow>) {
        let event = LiveEvent {
            event_type: LiveEventType::SkillUpdate,
            data: LiveEventData::SkillUpdate(SkillUpdateData {
                player_uid,
                skill_rows,
            }),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        };
        self.emit_event(event);
    }

    pub fn emit_encounter_reset(&self) {
        let event = LiveEvent {
            event_type: LiveEventType::EncounterReset,
            data: LiveEventData::EncounterUpdate(EncounterUpdateData {
                header_info: HeaderInfo::default(),
                is_paused: false,
            }),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        };
        self.emit_event(event);
    }

    pub fn emit_encounter_pause(&self, is_paused: bool) {
        let event_type = if is_paused {
            LiveEventType::EncounterPause
        } else {
            LiveEventType::EncounterResume
        };

        let event = LiveEvent {
            event_type,
            data: LiveEventData::EncounterUpdate(EncounterUpdateData {
                header_info: HeaderInfo::default(),
                is_paused,
            }),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        };
        self.emit_event(event);
    }

    pub fn emit_damage_update(
        &self,
        player_uid: i64,
        skill_uid: i32,
        damage_amount: u128,
        is_crit: bool,
        is_lucky: bool,
    ) {
        let event = LiveEvent {
            event_type: LiveEventType::NewDamage,
            data: LiveEventData::DamageUpdate(DamageUpdateData {
                player_uid,
                skill_uid,
                damage_amount,
                is_crit,
                is_lucky,
            }),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        };
        self.emit_event(event);
    }

    pub fn emit_heal_update(
        &self,
        player_uid: i64,
        skill_uid: i32,
        heal_amount: u128,
        is_crit: bool,
        is_lucky: bool,
    ) {
        let event = LiveEvent {
            event_type: LiveEventType::NewHeal,
            data: LiveEventData::HealUpdate(HealUpdateData {
                player_uid,
                skill_uid,
                heal_amount,
                is_crit,
                is_lucky,
            }),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        };
        self.emit_event(event);
    }

    pub fn should_emit_events(&self) -> bool {
        self.app_handle.is_some()
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

pub type EventManagerMutex = std::sync::Mutex<EventManager>;

fn prettify_name(player_uid: i64, local_player_uid: i64, player_name: &String) -> String {
    if player_uid == local_player_uid && player_name.is_empty() {
        String::from("You")
    } else if player_uid == local_player_uid && !player_name.is_empty() {
        format!("{player_name} (You)")
    } else {
        player_name.clone()
    }
}

fn nan_is_zero(value: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        0.0
    } else {
        value
    }
}

pub fn generate_player_row(
    entity_uid: i64,
    entity: &Entity,
    encounter: &Encounter,
) -> Option<PlayerRow> {
    let is_player = entity.entity_type == EEntityType::EntChar;
    let did_damage = !entity.skill_uid_to_dmg_skill.is_empty();

    if !is_player || !did_damage {
        return None;
    }

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    if encounter.total_dmg == 0 {
        return None;
    }

    #[allow(clippy::cast_precision_loss)]
    Some(PlayerRow {
        uid: entity_uid as u128,
        name: prettify_name(entity_uid, encounter.local_player_uid, &entity.name),
        class_name: class::get_class_name(entity.class_id),
        class_spec_name: class::get_class_spec(entity.class_spec),
        ability_score: entity.ability_score as u128,
        total_dmg: entity.total_dmg,
        dps: nan_is_zero(entity.total_dmg as f64 / time_elapsed_secs),
        dmg_pct: nan_is_zero(entity.total_dmg as f64 / encounter.total_dmg as f64 * 100.0),
        crit_rate: nan_is_zero(
            entity.crit_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0,
        ),
        crit_dmg_rate: nan_is_zero(
            entity.crit_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
        ),
        lucky_rate: nan_is_zero(
            entity.lucky_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0,
        ),
        lucky_dmg_rate: nan_is_zero(
            entity.lucky_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
        ),
        hits: entity.hits_dmg,
        hits_per_minute: nan_is_zero(entity.hits_dmg as f64 / time_elapsed_secs * 60.0),
    })
}

pub fn generate_skill_rows(entity: &Entity, time_elapsed_secs: f64) -> Vec<SkillRow> {
    let mut skill_rows = Vec::new();

    for (&skill_uid, skill) in &entity.skill_uid_to_dmg_skill {
        #[allow(clippy::cast_precision_loss)]
        let skill_row = SkillRow {
            name: Skill::get_skill_name(skill_uid),
            total_dmg: skill.total_value,
            dps: nan_is_zero(skill.total_value as f64 / time_elapsed_secs),
            dmg_pct: nan_is_zero(skill.total_value as f64 / entity.total_dmg as f64 * 100.0),
            crit_rate: nan_is_zero(skill.crit_hits as f64 / skill.hits as f64 * 100.0),
            crit_dmg_rate: nan_is_zero(
                skill.crit_total_value as f64 / skill.total_value as f64 * 100.0,
            ),
            lucky_rate: nan_is_zero(skill.lucky_hits as f64 / skill.hits as f64 * 100.0),
            lucky_dmg_rate: nan_is_zero(
                skill.lucky_total_value as f64 / skill.total_value as f64 * 100.0,
            ),
            hits: skill.hits,
            hits_per_minute: nan_is_zero(skill.hits as f64 / time_elapsed_secs * 60.0),
        };
        skill_rows.push(skill_row);
    }

    // Sort skills descending by damage dealt
    skill_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    skill_rows
}

pub fn generate_header_info(encounter: &Encounter) -> Option<HeaderInfo> {
    if encounter.total_dmg == 0 {
        return None;
    }

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    #[allow(clippy::cast_precision_loss)]
    Some(HeaderInfo {
        total_dps: nan_is_zero(encounter.total_dmg as f64 / time_elapsed_secs),
        total_dmg: encounter.total_dmg,
        elapsed_ms: time_elapsed_ms,
    })
}