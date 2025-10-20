use crate::live::commands_models::{HeaderInfo, PlayerRow, PlayersWindow, SkillRow, SkillsWindow};
use crate::live::opcodes_models::{class, Encounter, Entity, Skill};
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    Dps,
    Heal,
    Tanked,
}

#[derive(Debug)]
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

    pub fn emit_encounter_update(&self, header_info: HeaderInfo, is_paused: bool) {
        if let Some(app_handle) = &self.app_handle {
            let payload = EncounterUpdatePayload {
                header_info,
                is_paused,
            };
            match app_handle.emit("encounter-update", payload) {
                Ok(_) => {}, // trace!("Emitted encounter-update event"),
                Err(e) => error!("Failed to emit encounter-update event: {}", e),
            }
        }
    }

    pub fn emit_players_update(&self, metric_type: MetricType, players_window: PlayersWindow) {
        if let Some(app_handle) = &self.app_handle {
            let payload = PlayersUpdatePayload {
                metric_type,
                players_window,
            };
            if let Err(e) = app_handle.emit("players-update", payload) {
                error!("Failed to emit players-update event: {}", e);
            }
        }
    }

    pub fn emit_skills_update(&self, metric_type: MetricType, player_uid: i64, skills_window: SkillsWindow) {
        if let Some(app_handle) = &self.app_handle {
            let payload = SkillsUpdatePayload {
                metric_type,
                player_uid,
                skills_window,
            };
            match app_handle.emit("skills-update", payload) {
                Ok(_) => trace!("Emitted skills-update event for player {} ({})", player_uid, format!("{:?}", metric_type)),
                Err(e) => error!("Failed to emit skills-update event: {}", e),
            }
        }
    }

    pub fn emit_encounter_reset(&self) {
        if let Some(app_handle) = &self.app_handle {
            match app_handle.emit("reset-encounter", "") {
                Ok(_) => trace!("Emitted reset-encounter event"),
                Err(e) => error!("Failed to emit reset-encounter event: {}", e),
            }
        }
    }

    pub fn emit_encounter_pause(&self, is_paused: bool) {
        if let Some(app_handle) = &self.app_handle {
            match app_handle.emit("pause-encounter", is_paused) {
                Ok(_) => trace!("Emitted pause-encounter event: {}", is_paused),
                Err(e) => error!("Failed to emit pause-encounter event: {}", e),
            }
        }
    }

    pub fn should_emit_events(&self) -> bool {
        self.app_handle.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterUpdatePayload {
    pub header_info: HeaderInfo,
    pub is_paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayersUpdatePayload {
    pub metric_type: MetricType,
    pub players_window: PlayersWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsUpdatePayload {
    pub metric_type: MetricType,
    pub player_uid: i64,
    pub skills_window: SkillsWindow,
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

// Use an async RwLock for non-blocking access from async tasks
pub type EventManagerMutex = RwLock<EventManager>;

// Helper functions for generating data structures
pub fn generate_players_window_dps(encounter: &Encounter) -> PlayersWindow {
    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);

    let mut players_window = PlayersWindow {
        player_rows: Vec::new(),
    };

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    if encounter.total_dmg == 0 {
        return players_window;
    }

    for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
        if let Some(player_row) = generate_player_row(entity_uid, entity, encounter) {
            players_window.player_rows.push(player_row);
        }
    }

    // Sort players descending by damage dealt
    players_window.player_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    players_window
}

pub fn generate_players_window_heal(encounter: &Encounter) -> PlayersWindow {
    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);

    let mut players_window = PlayersWindow {
        player_rows: Vec::new(),
    };

    if encounter.total_heal == 0 {
        return players_window;
    }

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
        let is_player = entity.entity_type == EEntityType::EntChar;
        let did_heal = !entity.skill_uid_to_heal_skill.is_empty();

        if is_player && did_heal {
            #[allow(clippy::cast_precision_loss)]
            let heal_row = PlayerRow {
                uid: entity_uid as u128,
                name: prettify_name(entity_uid, encounter.local_player_uid, &entity.name),
                class_name: class::get_class_name(entity.class_id),
                class_spec_name: class::get_class_spec(entity.class_spec),
                ability_score: entity.ability_score as u128,
                total_dmg: entity.total_heal,
                dps: nan_is_zero(entity.total_heal as f64 / time_elapsed_secs),
                dmg_pct: nan_is_zero(
                    entity.total_heal as f64 / encounter.total_heal as f64 * 100.0,
                ),
                crit_rate: nan_is_zero(
                    entity.crit_hits_heal as f64 / entity.hits_heal as f64 * 100.0,
                ),
                crit_dmg_rate: nan_is_zero(
                    entity.crit_total_heal as f64 / entity.total_heal as f64 * 100.0,
                ),
                lucky_rate: nan_is_zero(
                    entity.lucky_hits_heal as f64 / entity.hits_heal as f64 * 100.0,
                ),
                lucky_dmg_rate: nan_is_zero(
                    entity.lucky_total_heal as f64 / entity.total_heal as f64 * 100.0,
                ),
                hits: entity.hits_heal,
                hits_per_minute: nan_is_zero(entity.hits_heal as f64 / time_elapsed_secs * 60.0),
            };
            players_window.player_rows.push(heal_row);
        }
    }

    // Sort players descending by heal dealt
    players_window.player_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    players_window
}

pub fn generate_players_window_tanked(encounter: &Encounter) -> PlayersWindow {
    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);

    let mut players_window = PlayersWindow {
        player_rows: Vec::new(),
    };

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    // Calculate total damage taken across all players
    let mut total_taken_all: u128 = 0;
    for entity in encounter.entity_uid_to_entity.values() {
        if entity.entity_type == EEntityType::EntChar {
            total_taken_all += entity.total_taken;
        }
    }

    if total_taken_all == 0 {
        return players_window;
    }

    for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
        let is_player = entity.entity_type == EEntityType::EntChar;
        let took_damage = !entity.skill_uid_to_taken_skill.is_empty();

        if is_player && took_damage {
            #[allow(clippy::cast_precision_loss)]
            let tanked_row = PlayerRow {
                uid: entity_uid as u128,
                name: prettify_name(entity_uid, encounter.local_player_uid, &entity.name),
                class_name: class::get_class_name(entity.class_id),
                class_spec_name: class::get_class_spec(entity.class_spec),
                ability_score: entity.ability_score as u128,
                total_dmg: entity.total_taken,
                dps: nan_is_zero(entity.total_taken as f64 / time_elapsed_secs),
                dmg_pct: nan_is_zero(
                    entity.total_taken as f64 / total_taken_all as f64 * 100.0,
                ),
                crit_rate: nan_is_zero(
                    entity.crit_hits_taken as f64 / entity.hits_taken as f64 * 100.0,
                ),
                crit_dmg_rate: nan_is_zero(
                    entity.crit_total_taken as f64 / entity.total_taken as f64 * 100.0,
                ),
                lucky_rate: nan_is_zero(
                    entity.lucky_hits_taken as f64 / entity.hits_taken as f64 * 100.0,
                ),
                lucky_dmg_rate: nan_is_zero(
                    entity.lucky_total_taken as f64 / entity.total_taken as f64 * 100.0,
                ),
                hits: entity.hits_taken,
                hits_per_minute: nan_is_zero(entity.hits_taken as f64 / time_elapsed_secs * 60.0),
            };
            players_window.player_rows.push(tanked_row);
        }
    }

    // Sort players descending by damage taken
    players_window.player_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    players_window
}

pub fn generate_skills_window_dps(encounter: &Encounter, player_uid: i64) -> Option<SkillsWindow> {
    let entity = encounter.entity_uid_to_entity.get(&player_uid)?;

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    // Player DPS Stats
    #[allow(clippy::cast_precision_loss)]
    let mut skills_window = SkillsWindow {
        curr_player: vec![PlayerRow {
            uid: player_uid as u128,
            name: prettify_name(player_uid, encounter.local_player_uid, &entity.name),
            class_name: class::get_class_name(entity.class_id),
            class_spec_name: class::get_class_spec(entity.class_spec),
            ability_score: entity.ability_score as u128,
            total_dmg: entity.total_dmg,
            dps: nan_is_zero(entity.total_dmg as f64 / time_elapsed_secs),
            dmg_pct: nan_is_zero(entity.total_dmg as f64 / encounter.total_dmg as f64 * 100.0),
            crit_rate: nan_is_zero(entity.crit_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0),
            crit_dmg_rate: nan_is_zero(
                entity.crit_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
            ),
            lucky_rate: nan_is_zero(entity.lucky_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0),
            lucky_dmg_rate: nan_is_zero(
                entity.lucky_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
            ),
            hits: entity.hits_dmg,
            hits_per_minute: nan_is_zero(entity.hits_dmg as f64 / time_elapsed_secs * 60.0),
        }],
        skill_rows: Vec::new(),
    };

    // Skills for this player
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
        skills_window.skill_rows.push(skill_row);
    }

    // Sort skills descending by damage dealt
    skills_window.skill_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Some(skills_window)
}

pub fn generate_skills_window_heal(encounter: &Encounter, player_uid: i64) -> Option<SkillsWindow> {
    let entity = encounter.entity_uid_to_entity.get(&player_uid)?;

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    // Player Heal Stats
    #[allow(clippy::cast_precision_loss)]
    let mut skills_window = SkillsWindow {
        curr_player: vec![PlayerRow {
            uid: player_uid as u128,
            name: prettify_name(player_uid, encounter.local_player_uid, &entity.name),
            class_name: class::get_class_name(entity.class_id),
            class_spec_name: class::get_class_spec(entity.class_spec),
            ability_score: entity.ability_score as u128,
            total_dmg: entity.total_heal,
            dps: nan_is_zero(entity.total_heal as f64 / time_elapsed_secs),
            dmg_pct: nan_is_zero(entity.total_heal as f64 / encounter.total_heal as f64 * 100.0),
            crit_rate: nan_is_zero(entity.crit_hits_heal as f64 / entity.hits_heal as f64 * 100.0),
            crit_dmg_rate: nan_is_zero(
                entity.crit_total_heal as f64 / entity.total_heal as f64 * 100.0,
            ),
            lucky_rate: nan_is_zero(
                entity.lucky_hits_heal as f64 / entity.hits_heal as f64 * 100.0,
            ),
            lucky_dmg_rate: nan_is_zero(
                entity.lucky_total_heal as f64 / entity.total_heal as f64 * 100.0,
            ),
            hits: entity.hits_heal,
            hits_per_minute: nan_is_zero(entity.hits_heal as f64 / time_elapsed_secs * 60.0),
        }],
        skill_rows: Vec::new(),
    };

    // Skills for this player
    for (&skill_uid, skill) in &entity.skill_uid_to_heal_skill {
        #[allow(clippy::cast_precision_loss)]
        let skill_row = SkillRow {
            name: Skill::get_skill_name(skill_uid),
            total_dmg: skill.total_value,
            dps: nan_is_zero(skill.total_value as f64 / time_elapsed_secs),
            dmg_pct: nan_is_zero(skill.total_value as f64 / entity.total_heal as f64 * 100.0),
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
        skills_window.skill_rows.push(skill_row);
    }

    // Sort skills descending by heal dealt
    skills_window.skill_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Some(skills_window)
}

pub fn generate_skills_window_tanked(encounter: &Encounter, player_uid: i64) -> Option<SkillsWindow> {
    let entity = encounter.entity_uid_to_entity.get(&player_uid)?;

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    // Player Tanked Stats
    #[allow(clippy::cast_precision_loss)]
    let mut skills_window = SkillsWindow {
        curr_player: vec![PlayerRow {
            uid: player_uid as u128,
            name: prettify_name(player_uid, encounter.local_player_uid, &entity.name),
            class_name: class::get_class_name(entity.class_id),
            class_spec_name: class::get_class_spec(entity.class_spec),
            ability_score: entity.ability_score as u128,
            total_dmg: entity.total_taken,
            dps: nan_is_zero(entity.total_taken as f64 / time_elapsed_secs),
            dmg_pct: 100.0, // Always 100% for the current player view
            crit_rate: nan_is_zero(entity.crit_hits_taken as f64 / entity.hits_taken as f64 * 100.0),
            crit_dmg_rate: nan_is_zero(
                entity.crit_total_taken as f64 / entity.total_taken as f64 * 100.0,
            ),
            lucky_rate: nan_is_zero(
                entity.lucky_hits_taken as f64 / entity.hits_taken as f64 * 100.0,
            ),
            lucky_dmg_rate: nan_is_zero(
                entity.lucky_total_taken as f64 / entity.total_taken as f64 * 100.0,
            ),
            hits: entity.hits_taken,
            hits_per_minute: nan_is_zero(entity.hits_taken as f64 / time_elapsed_secs * 60.0),
        }],
        skill_rows: Vec::new(),
    };

    // Skills for this player (damage taken from various sources)
    for (&skill_uid, skill) in &entity.skill_uid_to_taken_skill {
        #[allow(clippy::cast_precision_loss)]
        let skill_row = SkillRow {
            name: Skill::get_skill_name(skill_uid),
            total_dmg: skill.total_value,
            dps: nan_is_zero(skill.total_value as f64 / time_elapsed_secs),
            dmg_pct: nan_is_zero(skill.total_value as f64 / entity.total_taken as f64 * 100.0),
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
        skills_window.skill_rows.push(skill_row);
    }

    // Sort skills descending by damage taken
    skills_window.skill_rows.sort_by(|this_row, other_row| {
        other_row
            .total_dmg
            .partial_cmp(&this_row.total_dmg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Some(skills_window)
}

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
        crit_rate: nan_is_zero(entity.crit_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0),
        crit_dmg_rate: nan_is_zero(entity.crit_total_dmg as f64 / entity.total_dmg as f64 * 100.0),
        lucky_rate: nan_is_zero(entity.lucky_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0),
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
        fight_start_timestamp_ms: encounter.time_fight_start_ms,
    })
}
