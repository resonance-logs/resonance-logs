use crate::live::commands_models::{
    BossHealth, HeaderInfo, PlayerRow, PlayersWindow, SkillRow, SkillsWindow,
};
use crate::live::opcodes_models::{Encounter, Entity, Skill, class};
use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

/// Represents the type of metric being displayed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// Damage per second.
    Dps,
    /// Healing per second.
    Heal,
    /// Damage taken per second.
    Tanked,
}

/// Manages events and emits them to the frontend.
#[derive(Debug)]
pub struct EventManager {
    app_handle: Option<AppHandle>,
    dead_bosses: HashSet<i64>,
    // Map boss_uid -> boss_name for persisted marking
    dead_boss_names: HashMap<i64, String>,
}

impl EventManager {
    /// Creates a new `EventManager`.
    pub fn new() -> Self {
        Self {
            app_handle: None,
            dead_bosses: HashSet::new(),
            dead_boss_names: HashMap::new(),
        }
    }

    /// Initializes the `EventManager` with a Tauri app handle.
    ///
    /// # Arguments
    ///
    /// * `app_handle` - A handle to the Tauri application instance.
    pub fn initialize(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
        info!("Event manager initialized");
    }

    /// Emits an encounter update event.
    ///
    /// # Arguments
    ///
    /// * `header_info` - The header information for the encounter.
    /// * `is_paused` - Whether the encounter is paused.
    pub fn emit_encounter_update(&self, header_info: HeaderInfo, is_paused: bool) {
        if let Some(app_handle) = &self.app_handle {
            let payload = EncounterUpdatePayload {
                header_info,
                is_paused,
            };
            match app_handle.emit("encounter-update", payload) {
                Ok(_) => {} // trace!("Emitted encounter-update event"),
                Err(e) => error!("Failed to emit encounter-update event: {}", e),
            }
        }
    }

    /// Emits a players update event.
    ///
    /// # Arguments
    ///
    /// * `metric_type` - The type of metric being displayed.
    /// * `players_window` - The players window data.
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

    /// Emits a skills update event.
    ///
    /// # Arguments
    ///
    /// * `metric_type` - The type of metric being displayed.
    /// * `player_uid` - The UID of the player.
    /// * `skills_window` - The skills window data.
    pub fn emit_skills_update(
        &self,
        metric_type: MetricType,
        player_uid: i64,
        skills_window: SkillsWindow,
    ) {
        if let Some(app_handle) = &self.app_handle {
            let payload = SkillsUpdatePayload {
                metric_type,
                player_uid,
                skills_window,
            };
            if let Err(e) = app_handle.emit("skills-update", payload) {
                error!("Failed to emit skills-update event: {}", e);
            }
        }
    }

    /// Emits an encounter reset event.
    pub fn emit_encounter_reset(&self) {
        if let Some(app_handle) = &self.app_handle {
            match app_handle.emit("reset-encounter", "") {
                Ok(_) => trace!("Emitted reset-encounter event"),
                Err(e) => error!("Failed to emit reset-encounter event: {}", e),
            }
        }
    }

    /// Emits an encounter pause event.
    ///
    /// # Arguments
    ///
    /// * `is_paused` - Whether the encounter is paused.
    pub fn emit_encounter_pause(&self, is_paused: bool) {
        if let Some(app_handle) = &self.app_handle {
            match app_handle.emit("pause-encounter", is_paused) {
                Ok(_) => trace!("Emitted pause-encounter event: {}", is_paused),
                Err(e) => error!("Failed to emit pause-encounter event: {}", e),
            }
        }
    }

    /// Emits a scene change event.
    ///
    /// # Arguments
    ///
    /// * `scene_name` - The name of the new scene.
    pub fn emit_scene_change(&self, scene_name: String) {
        if let Some(app_handle) = &self.app_handle {
            let payload = SceneChangePayload { scene_name };
            match app_handle.emit("scene-change", payload) {
                Ok(_) => info!("Emitted scene-change event"),
                Err(e) => error!("Failed to emit scene-change event: {}", e),
            }
        }
    }

    /// Emits a boss death event.
    ///
    /// # Arguments
    ///
    /// * `boss_name` - The name of the boss that died.
    /// * `boss_uid` - The UID of the boss that died.
    pub fn emit_boss_death(&mut self, boss_name: String, boss_uid: i64) {
        // Only emit if we haven't already emitted for this boss
        if self.dead_bosses.insert(boss_uid) {
            // record the boss name for later persistence
            self.dead_boss_names.insert(boss_uid, boss_name.clone());
            if let Some(app_handle) = &self.app_handle {
                let payload = BossDeathPayload { boss_name };
                match app_handle.emit("boss-death", payload) {
                    Ok(_) => info!("Emitted boss-death event for {}", boss_uid),
                    Err(e) => error!("Failed to emit boss-death event: {}", e),
                }
            }
        }
    }

    /// Peek at dead boss names without consuming them.
    pub fn peek_dead_bosses(&self) -> Vec<String> {
        self.dead_boss_names.values().cloned().collect()
    }

    /// Drain and return any dead boss names that have been recorded by the event manager.
    /// This consumes the stored names and uids so they won't be double-persisted.
    pub fn take_dead_bosses(&mut self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for (_uid, name) in self.dead_boss_names.drain() {
            names.push(name);
        }
        // also clear uids set to keep parity
        self.dead_bosses.clear();
        names
    }

    /// Clears the list of dead bosses.
    pub fn clear_dead_bosses(&mut self) {
        self.dead_bosses.clear();
    }

    /// Returns whether the `EventManager` should emit events.
    pub fn should_emit_events(&self) -> bool {
        self.app_handle.is_some()
    }

    /// Returns a clone of the app handle for lock-free event emission.
    pub fn get_app_handle(&self) -> Option<AppHandle> {
        self.app_handle.clone()
    }

    /// Marks a boss as dead (used for deduplication).
    /// Returns true if this is a new death, false if already recorded.
    pub fn mark_boss_dead(&mut self, boss_uid: i64) -> bool {
        self.dead_bosses.insert(boss_uid)
    }
}

/// The payload for an encounter update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterUpdatePayload {
    /// The header information for the encounter.
    pub header_info: HeaderInfo,
    /// Whether the encounter is paused.
    pub is_paused: bool,
}

/// The payload for a players update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayersUpdatePayload {
    /// The type of metric being displayed.
    pub metric_type: MetricType,
    /// The players window data.
    pub players_window: PlayersWindow,
}

/// The payload for a skills update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsUpdatePayload {
    /// The type of metric being displayed.
    pub metric_type: MetricType,
    /// The UID of the player.
    pub player_uid: i64,
    /// The skills window data.
    pub skills_window: SkillsWindow,
}

/// The payload for a boss death event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BossDeathPayload {
    /// The name of the boss that died.
    pub boss_name: String,
}

/// The payload for a scene change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneChangePayload {
    /// The name of the new scene.
    pub scene_name: String,
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

// Use an async RwLock for non-blocking access from async tasks
pub type EventManagerMutex = RwLock<EventManager>;

// Helper: check if a target UID represents a boss entity
fn is_boss_target(encounter: &Encounter, target_uid: &i64) -> bool {
    encounter
        .entity_uid_to_entity
        .get(target_uid)
        .map(|e| e.is_boss())
        .unwrap_or(false)
}

// Helper functions for generating data structures
pub fn generate_players_window_dps(encounter: &Encounter, boss_only: bool) -> PlayersWindow {
    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);

    let mut players_window = PlayersWindow {
        player_rows: Vec::new(),
    };

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    let total_scope_dmg: u128 = if boss_only {
        encounter
            .entity_uid_to_entity
            .iter()
            .filter(|(_, e)| e.entity_type == EEntityType::EntChar)
            .map(|(_, e)| {
                e.dmg_to_target
                    .iter()
                    .filter(|(tuid, _)| is_boss_target(encounter, tuid))
                    .map(|(_, v)| *v)
                    .sum::<u128>()
            })
            .sum()
    } else {
        encounter.total_dmg
    };

    if total_scope_dmg == 0 {
        return players_window;
    }

    for (&entity_uid, entity) in &encounter.entity_uid_to_entity {
        if let Some(player_row) = generate_player_row_filtered(
            entity_uid,
            entity,
            encounter,
            boss_only,
            total_scope_dmg,
            time_elapsed_secs,
        ) {
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
                // Extended attributes from Stage 4
                rank_level: entity.rank_level(),
                current_hp: entity.hp(),
                max_hp: entity.max_hp(),
                crit_stat: entity.crit(),
                lucky_stat: entity.lucky(),
                haste: entity.haste(),
                mastery: entity.mastery(),
                element_flag: entity
                    .get_attr(crate::live::opcodes_models::AttrType::ElementFlag)
                    .and_then(|v| v.as_int()),
                energy_flag: entity
                    .get_attr(crate::live::opcodes_models::AttrType::EnergyFlag)
                    .and_then(|v| v.as_int()),
                reduction_level: entity.reduction_level(),
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
                dmg_pct: nan_is_zero(entity.total_taken as f64 / total_taken_all as f64 * 100.0),
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
                // Extended attributes from Stage 4
                rank_level: entity.rank_level(),
                current_hp: entity.hp(),
                max_hp: entity.max_hp(),
                crit_stat: entity.crit(),
                lucky_stat: entity.lucky(),
                haste: entity.haste(),
                mastery: entity.mastery(),
                element_flag: entity
                    .get_attr(crate::live::opcodes_models::AttrType::ElementFlag)
                    .and_then(|v| v.as_int()),
                energy_flag: entity
                    .get_attr(crate::live::opcodes_models::AttrType::EnergyFlag)
                    .and_then(|v| v.as_int()),
                reduction_level: entity.reduction_level(),
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

pub fn generate_skills_window_dps(
    encounter: &Encounter,
    player_uid: i64,
    boss_only: bool,
) -> Option<SkillsWindow> {
    let entity = encounter.entity_uid_to_entity.get(&player_uid)?;

    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);
    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    // Compute encounter and player totals within scope
    let total_scope_dmg: u128 = if boss_only {
        encounter
            .entity_uid_to_entity
            .iter()
            .filter(|(_, e)| e.entity_type == EEntityType::EntChar)
            .map(|(_, e)| {
                e.dmg_to_target
                    .iter()
                    .filter(|(tuid, _)| is_boss_target(encounter, tuid))
                    .map(|(_, v)| *v)
                    .sum::<u128>()
            })
            .sum()
    } else {
        encounter.total_dmg
    };

    let player_total: u128 = if boss_only {
        entity
            .dmg_to_target
            .iter()
            .filter(|(tuid, _)| is_boss_target(encounter, tuid))
            .map(|(_, v)| *v)
            .sum()
    } else {
        entity.total_dmg
    };

    // Player DPS Stats
    #[allow(clippy::cast_precision_loss)]
    let mut skills_window = SkillsWindow {
        curr_player: vec![PlayerRow {
            uid: player_uid as u128,
            name: prettify_name(player_uid, encounter.local_player_uid, &entity.name),
            class_name: class::get_class_name(entity.class_id),
            class_spec_name: class::get_class_spec(entity.class_spec),
            ability_score: entity.ability_score as u128,
            total_dmg: player_total,
            dps: nan_is_zero(player_total as f64 / time_elapsed_secs),
            dmg_pct: if total_scope_dmg == 0 {
                0.0
            } else {
                nan_is_zero(player_total as f64 / total_scope_dmg as f64 * 100.0)
            },
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
            // Extended attributes from Stage 4
            rank_level: entity.rank_level(),
            current_hp: entity.hp(),
            max_hp: entity.max_hp(),
            crit_stat: entity.crit(),
            lucky_stat: entity.lucky(),
            haste: entity.haste(),
            mastery: entity.mastery(),
            element_flag: entity
                .get_attr(crate::live::opcodes_models::AttrType::ElementFlag)
                .and_then(|v| v.as_int()),
            energy_flag: entity
                .get_attr(crate::live::opcodes_models::AttrType::EnergyFlag)
                .and_then(|v| v.as_int()),
            reduction_level: entity.reduction_level(),
        }],
        skill_rows: Vec::new(),
    };

    // Skills for this player
    for (&skill_uid, skill) in &entity.skill_uid_to_dmg_skill {
        let skill_total: u128 = if boss_only {
            entity
                .skill_dmg_to_target
                .get(&skill_uid)
                .map(|m| {
                    m.iter()
                        .filter(|(tuid, _)| is_boss_target(encounter, tuid))
                        .map(|(_, v)| *v)
                        .sum()
                })
                .unwrap_or(0)
        } else {
            skill.total_value
        };
        #[allow(clippy::cast_precision_loss)]
        let skill_row = SkillRow {
            name: Skill::get_skill_name(skill_uid),
            total_dmg: skill_total,
            dps: nan_is_zero(skill_total as f64 / time_elapsed_secs),
            dmg_pct: if player_total == 0 {
                0.0
            } else {
                nan_is_zero(skill_total as f64 / player_total as f64 * 100.0)
            },
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
            // Extended attributes from Stage 4
            rank_level: entity.rank_level(),
            current_hp: entity.hp(),
            max_hp: entity.max_hp(),
            crit_stat: entity.crit(),
            lucky_stat: entity.lucky(),
            haste: entity.haste(),
            mastery: entity.mastery(),
            element_flag: entity
                .get_attr(crate::live::opcodes_models::AttrType::ElementFlag)
                .and_then(|v| v.as_int()),
            energy_flag: entity
                .get_attr(crate::live::opcodes_models::AttrType::EnergyFlag)
                .and_then(|v| v.as_int()),
            reduction_level: entity.reduction_level(),
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

pub fn generate_skills_window_tanked(
    encounter: &Encounter,
    player_uid: i64,
) -> Option<SkillsWindow> {
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
            // Extended attributes from Stage 4
            rank_level: entity.rank_level(),
            current_hp: entity.hp(),
            max_hp: entity.max_hp(),
            crit_stat: entity.crit(),
            lucky_stat: entity.lucky(),
            haste: entity.haste(),
            mastery: entity.mastery(),
            element_flag: entity
                .get_attr(crate::live::opcodes_models::AttrType::ElementFlag)
                .and_then(|v| v.as_int()),
            energy_flag: entity
                .get_attr(crate::live::opcodes_models::AttrType::EnergyFlag)
                .and_then(|v| v.as_int()),
            reduction_level: entity.reduction_level(),
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
    // If entity name is empty, try to get it from the database
    let effective_name = if player_name.is_empty() {
        crate::live::player_names::PlayerNames::get_name_by_uid(player_uid)
            .unwrap_or_else(|| String::new())
    } else {
        player_name.clone()
    };

    if player_uid == local_player_uid && effective_name.is_empty() {
        String::from("You")
    } else if player_uid == local_player_uid && !effective_name.is_empty() {
        format!("{effective_name} (You)")
    } else if effective_name.is_empty() {
        format!("#{player_uid}")
    } else {
        effective_name
    }
}

fn nan_is_zero(value: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        0.0
    } else {
        value
    }
}

pub fn generate_player_row_filtered(
    entity_uid: i64,
    entity: &Entity,
    encounter: &Encounter,
    boss_only: bool,
    total_scope_dmg: u128,
    time_elapsed_secs: f64,
) -> Option<PlayerRow> {
    let is_player = entity.entity_type == EEntityType::EntChar;
    let did_damage = !entity.skill_uid_to_dmg_skill.is_empty();

    if !is_player || !did_damage {
        return None;
    }

    let entity_total: u128 = if boss_only {
        entity
            .dmg_to_target
            .iter()
            .filter(|(tuid, _)| is_boss_target(encounter, tuid))
            .map(|(_, v)| *v)
            .sum()
    } else {
        entity.total_dmg
    };
    if total_scope_dmg == 0 {
        return None;
    }

    #[allow(clippy::cast_precision_loss)]
    Some(PlayerRow {
        uid: entity_uid as u128,
        name: prettify_name(entity_uid, encounter.local_player_uid, &entity.name),
        class_name: class::get_class_name(entity.class_id),
        class_spec_name: class::get_class_spec(entity.class_spec),
        ability_score: entity.ability_score as u128,
        total_dmg: entity_total,
        dps: nan_is_zero(entity_total as f64 / time_elapsed_secs),
        dmg_pct: nan_is_zero(entity_total as f64 / total_scope_dmg as f64 * 100.0),
        crit_rate: nan_is_zero(entity.crit_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0),
        crit_dmg_rate: nan_is_zero(entity.crit_total_dmg as f64 / entity.total_dmg as f64 * 100.0),
        lucky_rate: nan_is_zero(entity.lucky_hits_dmg as f64 / entity.hits_dmg as f64 * 100.0),
        lucky_dmg_rate: nan_is_zero(
            entity.lucky_total_dmg as f64 / entity.total_dmg as f64 * 100.0,
        ),
        hits: entity.hits_dmg,
        hits_per_minute: nan_is_zero(entity.hits_dmg as f64 / time_elapsed_secs * 60.0),
        // Extended attributes from Stage 4
        rank_level: entity.rank_level(),
        current_hp: entity.hp(),
        max_hp: entity.max_hp(),
        crit_stat: entity.crit(),
        lucky_stat: entity.lucky(),
        haste: entity.haste(),
        mastery: entity.mastery(),
        element_flag: entity
            .get_attr(crate::live::opcodes_models::AttrType::ElementFlag)
            .and_then(|v| v.as_int()),
        energy_flag: entity
            .get_attr(crate::live::opcodes_models::AttrType::EnergyFlag)
            .and_then(|v| v.as_int()),
        reduction_level: entity.reduction_level(),
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

pub fn generate_header_info(
    encounter: &Encounter,
    boss_only: bool,
) -> Option<(HeaderInfo, Vec<(i64, String)>)> {
    let time_elapsed_ms = encounter
        .time_last_combat_packet_ms
        .saturating_sub(encounter.time_fight_start_ms);

    #[allow(clippy::cast_precision_loss)]
    let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

    let total_scope_dmg: u128 = if boss_only {
        encounter
            .entity_uid_to_entity
            .iter()
            .filter(|(_, e)| e.entity_type == EEntityType::EntChar)
            .map(|(_, e)| {
                e.dmg_to_target
                    .iter()
                    .filter(|(tuid, _)| is_boss_target(encounter, tuid))
                    .map(|(_, v)| *v)
                    .sum::<u128>()
            })
            .sum()
    } else {
        encounter.total_dmg
    };

    // Calculate team DPS for boss death detection
    #[allow(clippy::cast_precision_loss)]
    let team_dps = nan_is_zero(total_scope_dmg as f64 / time_elapsed_secs);

    let mut dead_bosses: Vec<(i64, String)> = Vec::new();
    let mut bosses: Vec<BossHealth> = encounter
        .entity_uid_to_entity
        .iter()
        .filter_map(|(&uid, entity)| {
            if entity.is_boss() {
                let current_hp = entity.hp();
                let max_hp = entity.max_hp();

                // Filter out bosses without HP attributes (cleared after reset)
                if current_hp.is_none() && max_hp.is_none() {
                    return None;
                }

                let name = if !entity.name.is_empty() {
                    entity.name.clone()
                } else if let Some(packet_name) = &entity.monster_name_packet {
                    packet_name.clone()
                } else {
                    format!("Boss {uid}")
                };

                // Boss death detection: if boss has <5% HP and team DPS is high enough, assume boss is dead
                let is_dead = if let (Some(curr_hp), Some(max_hp_val)) = (current_hp, max_hp) {
                    if max_hp_val > 0 {
                        let hp_percent = (curr_hp as f64 / max_hp_val as f64) * 100.0;
                        // If boss is below 5% HP and team DPS is at least 10k, assume dead
                        hp_percent < 5.0 && team_dps >= 5000.0
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_dead {
                    dead_bosses.push((uid, name.clone()));
                }

                Some(BossHealth {
                    uid,
                    name,
                    // Set HP to 0 if boss is detected as dead
                    current_hp: if is_dead { Some(0) } else { current_hp },
                    max_hp,
                })
            } else {
                None
            }
        })
        .collect();

    bosses.sort_by_key(|boss| boss.uid);

    #[allow(clippy::cast_precision_loss)]
    Some((
        HeaderInfo {
            total_dps: team_dps,
            total_dmg: total_scope_dmg,
            elapsed_ms: time_elapsed_ms,
            fight_start_timestamp_ms: encounter.time_fight_start_ms,
            bosses,
            scene_id: encounter.current_scene_id,
            scene_name: encounter.current_scene_name.clone(),
            current_segment_type: None,
            current_segment_name: None,
        },
        dead_bosses,
    ))
}
