use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema as sch;

/// Represents a row in the `detailed_playerdata` table.
#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = sch::detailed_playerdata, primary_key(player_id))]
pub struct DetailedPlayerDataRow {
    /// The unique ID of the player.
    pub player_id: i64,
    /// The timestamp of when the player data was last seen.
    pub last_seen_ms: i64,
    /// The serialized `CharSerialize` payload for the player.
    pub char_serialize_json: String,
    /// The serialized profession list associated with the player.
    pub profession_list_json: Option<String>,
    /// The serialized talent node identifiers for the player.
    pub talent_node_ids_json: Option<String>,
}

/// Represents a new row to insert into the `detailed_playerdata` table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::detailed_playerdata)]
pub struct NewDetailedPlayerData<'a> {
    /// The unique ID of the player.
    pub player_id: i64,
    /// The timestamp of when the player data was last seen.
    pub last_seen_ms: i64,
    /// The serialized `CharSerialize` payload for the player.
    pub char_serialize_json: &'a str,
    /// The serialized profession list associated with the player.
    pub profession_list_json: Option<&'a str>,
    /// The serialized talent node identifiers for the player.
    pub talent_node_ids_json: Option<&'a str>,
}

/// Represents an update to an existing row in the `detailed_playerdata` table.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = sch::detailed_playerdata)]
pub struct UpdateDetailedPlayerData<'a> {
    /// The timestamp of when the player data was last seen.
    pub last_seen_ms: i64,
    /// The serialized `CharSerialize` payload for the player.
    pub char_serialize_json: &'a str,
    /// The serialized profession list associated with the player.
    pub profession_list_json: Option<&'a str>,
    /// The serialized talent node identifiers for the player.
    pub talent_node_ids_json: Option<&'a str>,
}

/// Represents a row in the `entities` table.
#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = sch::entities, primary_key(entity_id))]
pub struct EntityRow {
    /// The unique ID of the entity.
    pub entity_id: i64,
    /// The name of the entity.
    pub name: Option<String>,
    /// The class ID of the entity.
    pub class_id: Option<i32>,
    /// The class spec of the entity.
    pub class_spec: Option<i32>,
    /// The ability score of the entity.
    pub ability_score: Option<i32>,
    /// The level of the entity.
    pub level: Option<i32>,
    /// The timestamp of when the entity was first seen, in milliseconds since the Unix epoch.
    pub first_seen_ms: Option<i64>,
    /// The timestamp of when the entity was last seen, in milliseconds since the Unix epoch.
    pub last_seen_ms: Option<i64>,
    /// The attributes of the entity.
    pub attributes: Option<String>,
}

/// Represents a new entity to be inserted into the `entities` table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::entities)]
pub struct NewEntity<'a> {
    /// The unique ID of the entity.
    pub entity_id: i64,
    /// The name of the entity.
    pub name: Option<&'a str>,
    /// The class ID of the entity.
    pub class_id: Option<i32>,
    /// The class spec of the entity.
    pub class_spec: Option<i32>,
    /// The ability score of the entity.
    pub ability_score: Option<i32>,
    /// The level of the entity.
    pub level: Option<i32>,
    /// The timestamp of when the entity was first seen, in milliseconds since the Unix epoch.
    pub first_seen_ms: Option<i64>,
    /// The timestamp of when the entity was last seen, in milliseconds since the Unix epoch.
    pub last_seen_ms: Option<i64>,
    /// The attributes of the entity.
    pub attributes: Option<&'a str>,
}

/// Represents an update to an entity in the `entities` table.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = sch::entities)]
pub struct UpdateEntity<'a> {
    /// The name of the entity.
    pub name: Option<&'a str>,
    /// The class ID of the entity.
    pub class_id: Option<i32>,
    /// The class spec of the entity.
    pub class_spec: Option<i32>,
    /// The ability score of the entity.
    pub ability_score: Option<i32>,
    /// The level of the entity.
    pub level: Option<i32>,
    /// The timestamp of when the entity was last seen, in milliseconds since the Unix epoch.
    pub last_seen_ms: Option<i64>,
    /// The attributes of the entity.
    pub attributes: Option<&'a str>,
}

/// Represents a row in the `encounters` table.
#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = sch::encounters)]
pub struct EncounterRow {
    /// The unique ID of the encounter.
    pub id: i32,
    /// The timestamp of when the encounter started, in milliseconds since the Unix epoch.
    pub started_at_ms: i64,
    /// The timestamp of when the encounter ended, in milliseconds since the Unix epoch.
    pub ended_at_ms: Option<i64>,
    /// The ID of the local player.
    pub local_player_id: Option<i64>,
    /// The total damage dealt in the encounter.
    pub total_dmg: Option<i64>,
    /// The total healing done in the encounter.
    pub total_heal: Option<i64>,
    /// The ID of the scene where the encounter took place.
    pub scene_id: Option<i32>,
    /// The name of the scene where the encounter took place.
    pub scene_name: Option<String>,
    /// The duration of the encounter in seconds.
    pub duration: f64,
    /// When this encounter was uploaded to the website (ms since epoch).
    pub uploaded_at_ms: Option<i64>,
}

/// Represents a new encounter to be inserted into the `encounters` table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::encounters)]
pub struct NewEncounter {
    /// The timestamp of when the encounter started, in milliseconds since the Unix epoch.
    pub started_at_ms: i64,
    /// The timestamp of when the encounter ended, in milliseconds since the Unix epoch.
    pub ended_at_ms: Option<i64>,
    /// The ID of the local player.
    pub local_player_id: Option<i64>,
    /// The total damage dealt in the encounter.
    pub total_dmg: Option<i64>,
    /// The total healing done in the encounter.
    pub total_heal: Option<i64>,
    /// The ID of the scene where the encounter took place.
    pub scene_id: Option<i32>,
    /// The name of the scene where the encounter took place.
    pub scene_name: Option<String>,
    /// The duration of the encounter in seconds.
    pub duration: f64,
}

/// Represents a row in the `damage_events` table.
// Raw per-event structs (damage_events / heal_events) removed — storage is now aggregate-only.
// The following Diesel structs were intentionally removed as part of the irreversible
// schema change: DamageEventRow, NewDamageEvent, HealEventRow, NewHealEvent.

/// Represents a row in the `death_events` table.
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = sch::death_events, belongs_to(EncounterRow, foreign_key = encounter_id))]
pub struct DeathEventRow {
    /// The unique ID of the death event.
    pub id: i32,
    /// The ID of the encounter this event belongs to.
    pub encounter_id: i32,
    /// The timestamp of the event, in milliseconds since the Unix epoch.
    pub timestamp_ms: i64,
    /// The ID of the actor who died.
    pub actor_id: i64,
    /// The ID of the killer (if known).
    pub killer_id: Option<i64>,
    /// The ID of the skill that caused the death (if known).
    pub skill_id: Option<i32>,
    /// Whether the actor was the local player.
    pub is_local_player: i32,
    /// The attempt index this death occurred in.
    pub attempt_index: Option<i32>,
}

/// Represents a new death event to be inserted into the `death_events` table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::death_events)]
pub struct NewDeathEvent {
    /// The ID of the encounter this event belongs to.
    pub encounter_id: i32,
    /// The timestamp of the event, in milliseconds since the Unix epoch.
    pub timestamp_ms: i64,
    /// The ID of the actor who died.
    pub actor_id: i64,
    /// The ID of the killer (if known).
    pub killer_id: Option<i64>,
    /// The ID of the skill that caused the death (if known).
    pub skill_id: Option<i32>,
    /// Whether the actor was the local player.
    pub is_local_player: i32,
    /// The attempt index this death occurred in.
    pub attempt_index: Option<i32>,
}

/// Represents a row in the `actor_encounter_stats` table.
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = sch::actor_encounter_stats, primary_key(encounter_id, actor_id))]
#[diesel(belongs_to(EncounterRow, foreign_key = encounter_id))]
pub struct ActorEncounterStatRow {
    /// The ID of the encounter.
    pub encounter_id: i32,
    /// The ID of the actor.
    pub actor_id: i64,
    /// The name of the actor.
    pub name: Option<String>,
    /// The class ID of the actor.
    pub class_id: Option<i32>,
    /// The class spec of the actor.
    pub class_spec: Option<i32>,
    /// The ability score of the actor.
    pub ability_score: Option<i32>,
    /// The level of the actor.
    pub level: Option<i32>,
    /// The total damage dealt by the actor.
    pub damage_dealt: i64,
    /// The total healing done by the actor.
    pub heal_dealt: i64,
    /// The total damage taken by the actor.
    pub damage_taken: i64,
    /// The number of hits dealt by the actor.
    pub hits_dealt: i64,
    /// The number of hits healed by the actor.
    pub hits_heal: i64,
    /// The number of hits taken by the actor.
    pub hits_taken: i64,
    /// The number of critical hits dealt by the actor.
    pub crit_hits_dealt: i64,
    /// The number of critical hits healed by the actor.
    pub crit_hits_heal: i64,
    /// The number of critical hits taken by the actor.
    pub crit_hits_taken: i64,
    /// The number of lucky hits dealt by the actor.
    pub lucky_hits_dealt: i64,
    /// The number of lucky hits healed by the actor.
    pub lucky_hits_heal: i64,
    /// The number of lucky hits taken by the actor.
    pub lucky_hits_taken: i64,
    /// The total critical damage dealt by the actor.
    pub crit_total_dealt: i64,
    /// The total critical healing done by the actor.
    pub crit_total_heal: i64,
    /// The total critical damage taken by the actor.
    pub crit_total_taken: i64,
    /// The total lucky damage dealt by the actor.
    pub lucky_total_dealt: i64,
    /// The total lucky healing done by the actor.
    pub lucky_total_heal: i64,
    /// The total lucky damage taken by the actor.
    pub lucky_total_taken: i64,
    /// The total damage dealt to bosses by the actor.
    pub boss_damage_dealt: i64,
    /// The number of hits dealt to bosses by the actor.
    pub boss_hits_dealt: i64,
    /// The number of critical hits dealt to bosses by the actor.
    pub boss_crit_hits_dealt: i64,
    /// The number of lucky hits dealt to bosses by the actor.
    pub boss_lucky_hits_dealt: i64,
    /// The total critical damage dealt to bosses by the actor.
    pub boss_crit_total_dealt: i64,
    /// The total lucky damage dealt to bosses by the actor.
    pub boss_lucky_total_dealt: i64,
    /// The number of revives for the actor during the encounter.
    pub revives: i64,
    /// The average DPS snapshot for the actor during the encounter.
    pub dps: f64,
    /// The encounter duration in seconds used for the DPS snapshot.
    pub duration: f64,
    /// Whether the actor is a player.
    pub is_player: i32,
    /// Whether the actor is the local player.
    pub is_local_player: i32,
    /// The attributes of the actor.
    pub attributes: Option<String>,
}

/// Represents a new actor encounter stat to be inserted into the `actor_encounter_stats` table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::actor_encounter_stats)]
pub struct NewActorEncounterStat {
    /// The ID of the encounter.
    pub encounter_id: i32,
    /// The ID of the actor.
    pub actor_id: i64,
    /// The name of the actor.
    pub name: Option<String>,
    /// The class ID of the actor.
    pub class_id: Option<i32>,
    /// The class spec of the actor.
    pub class_spec: Option<i32>,
    /// The ability score of the actor.
    pub ability_score: Option<i32>,
    /// The level of the actor.
    pub level: Option<i32>,
    /// The total damage dealt by the actor.
    pub damage_dealt: i64,
    /// The total healing done by the actor.
    pub heal_dealt: i64,
    /// The total damage taken by the actor.
    pub damage_taken: i64,
    /// The number of hits dealt by the actor.
    pub hits_dealt: i64,
    /// The number of hits healed by the actor.
    pub hits_heal: i64,
    /// The number of hits taken by the actor.
    pub hits_taken: i64,
    /// The number of critical hits dealt by the actor.
    pub crit_hits_dealt: i64,
    /// The number of critical hits healed by the actor.
    pub crit_hits_heal: i64,
    /// The number of critical hits taken by the actor.
    pub crit_hits_taken: i64,
    /// The number of lucky hits dealt by the actor.
    pub lucky_hits_dealt: i64,
    /// The number of lucky hits healed by the actor.
    pub lucky_hits_heal: i64,
    /// The number of lucky hits taken by the actor.
    pub lucky_hits_taken: i64,
    /// The total critical damage dealt by the actor.
    pub crit_total_dealt: i64,
    /// The total critical healing done by the actor.
    pub crit_total_heal: i64,
    /// The total critical damage taken by the actor.
    pub crit_total_taken: i64,
    /// The total lucky damage dealt by the actor.
    pub lucky_total_dealt: i64,
    /// The total lucky healing done by the actor.
    pub lucky_total_heal: i64,
    /// The total lucky damage taken by the actor.
    pub lucky_total_taken: i64,
    /// The total damage dealt to bosses by the actor.
    pub boss_damage_dealt: i64,
    /// The number of hits dealt to bosses by the actor.
    pub boss_hits_dealt: i64,
    /// The number of critical hits dealt to bosses by the actor.
    pub boss_crit_hits_dealt: i64,
    /// The number of lucky hits dealt to bosses by the actor.
    pub boss_lucky_hits_dealt: i64,
    /// The total critical damage dealt to bosses by the actor.
    pub boss_crit_total_dealt: i64,
    /// The total lucky damage dealt to bosses by the actor.
    pub boss_lucky_total_dealt: i64,
    /// The number of revives for the actor during the encounter.
    pub revives: i64,
    /// The average DPS snapshot for the actor during the encounter.
    pub dps: f64,
    /// The encounter duration in seconds used for the DPS snapshot.
    pub duration: f64,
}
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(
    table_name = sch::damage_skill_stats,
    primary_key(encounter_id, attacker_id, defender_id, skill_id)
)]
#[diesel(belongs_to(EncounterRow, foreign_key = encounter_id))]
pub struct DamageSkillStatRow {
    /// The ID of the encounter.
    pub encounter_id: i32,
    /// The ID of the attacker.
    pub attacker_id: i64,
    /// The ID of the defender.
    pub defender_id: Option<i64>,
    /// The ID of the skill used.
    pub skill_id: i32,
    /// The number of hits.
    pub hits: i32,
    /// The total value of the damage.
    pub total_value: i64,
    /// The number of critical hits.
    pub crit_hits: i32,
    /// The number of lucky hits.
    pub lucky_hits: i32,
    /// The total critical damage.
    pub crit_total: i64,
    /// The total lucky damage.
    pub lucky_total: i64,
    /// The total HP lost.
    pub hp_loss_total: i64,
    /// The total shield lost.
    pub shield_loss_total: i64,
    /// The details of the hits.
    pub hit_details: String,
    /// The name of the monster.
    pub monster_name: Option<String>,
}

/// Represents a new damage skill stat to be inserted into the `damage_skill_stats` table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::damage_skill_stats)]
pub struct NewDamageSkillStat {
    /// The ID of the encounter.
    pub encounter_id: i32,
    /// The ID of the attacker.
    pub attacker_id: i64,
    /// The ID of the defender.
    pub defender_id: Option<i64>,
    /// The ID of the skill used.
    pub skill_id: i32,
    /// The number of hits.
    pub hits: i32,
    /// The total value of the damage.
    pub total_value: i64,
    /// The number of critical hits.
    pub crit_hits: i32,
    /// The number of lucky hits.
    pub lucky_hits: i32,
    /// The total critical damage.
    pub crit_total: i64,
    /// The total lucky damage.
    pub lucky_total: i64,
    /// The total HP lost.
    pub hp_loss_total: i64,
    /// The total shield lost.
    pub shield_loss_total: i64,
    /// The details of the hits.
    pub hit_details: String,
    /// The name of the monster.
    pub monster_name: Option<String>,
}

/// Represents a row in the `heal_skill_stats` table.
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = sch::heal_skill_stats, primary_key(encounter_id, healer_id, target_id, skill_id))]
#[diesel(belongs_to(EncounterRow, foreign_key = encounter_id))]
pub struct HealSkillStatRow {
    /// The ID of the encounter.
    pub encounter_id: i32,
    /// The ID of the healer.
    pub healer_id: i64,
    /// The ID of the target.
    pub target_id: Option<i64>,
    /// The ID of the skill used.
    pub skill_id: i32,
    /// The number of hits.
    pub hits: i32,
    /// The total value of the heal.
    pub total_value: i64,
    /// The number of critical hits.
    pub crit_hits: i32,
    /// The number of lucky hits.
    pub lucky_hits: i32,
    /// The total critical heal.
    pub crit_total: i64,
    /// The total lucky heal.
    pub lucky_total: i64,
    /// The details of the heals.
    pub heal_details: String,
    /// The name of the monster.
    pub monster_name: Option<String>,
}

/// Represents a new heal skill stat to be inserted into the `heal_skill_stats` table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::heal_skill_stats)]
pub struct NewHealSkillStat {
    /// The ID of the encounter.
    pub encounter_id: i32,
    /// The ID of the healer.
    pub healer_id: i64,
    /// The ID of the target.
    pub target_id: Option<i64>,
    /// The ID of the skill used.
    pub skill_id: i32,
    /// The number of hits.
    pub hits: i32,
    /// The total value of the heal.
    pub total_value: i64,
    /// The number of critical hits.
    pub crit_hits: i32,
    /// The number of lucky hits.
    pub lucky_hits: i32,
    /// The total critical heal.
    pub crit_total: i64,
    /// The total lucky heal.
    pub lucky_total: i64,
    /// The details of the heals.
    pub heal_details: String,
    /// The name of the monster.
    pub monster_name: Option<String>,
}

/// Represents a row in the `encounter_bosses` table.
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = sch::encounter_bosses, primary_key(encounter_id, monster_name))]
#[diesel(belongs_to(EncounterRow, foreign_key = encounter_id))]
pub struct EncounterBossRow {
    /// The ID of the encounter.
    pub encounter_id: i32,
    /// The name of the monster.
    pub monster_name: String,
    /// The number of hits.
    pub hits: i32,
    /// The total damage dealt to the boss.
    pub total_damage: i64,
    /// The maximum HP of the boss.
    pub max_hp: Option<i64>,
    /// Whether the boss was defeated.
    pub is_defeated: i32,
}

/// Represents a new encounter boss to be inserted into the `encounter_bosses` table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::encounter_bosses)]
pub struct NewEncounterBoss {
    /// The ID of the encounter.
    pub encounter_id: i32,
    /// The name of the monster.
    pub monster_name: String,
    /// The number of hits.
    pub hits: i32,
    /// The total damage dealt to the boss.
    pub total_damage: i64,
    /// The maximum HP of the boss.
    pub max_hp: Option<i64>,
    /// Whether the boss was defeated.
    pub is_defeated: Option<i32>,
}

/// Represents a row in the `attempts` table.
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = sch::attempts, belongs_to(EncounterRow, foreign_key = encounter_id))]
pub struct AttemptRow {
    /// The unique ID of the attempt.
    pub id: i32,
    /// The ID of the encounter this attempt belongs to.
    pub encounter_id: i32,
    /// The attempt index (1-based).
    pub attempt_index: i32,
    /// The timestamp of when the attempt started, in milliseconds since the Unix epoch.
    pub started_at_ms: i64,
    /// The timestamp of when the attempt ended, in milliseconds since the Unix epoch.
    pub ended_at_ms: Option<i64>,
    /// The reason for the attempt split ('wipe', 'hp_rollback', 'manual').
    pub reason: String,
    /// The boss HP at the start of the attempt.
    pub boss_hp_start: Option<i64>,
    /// The boss HP at the end of the attempt.
    pub boss_hp_end: Option<i64>,
    /// The total number of deaths in this attempt.
    pub total_deaths: i32,
}

/// Represents a new attempt to be inserted into the `attempts` table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::attempts)]
pub struct NewAttempt {
    /// The ID of the encounter this attempt belongs to.
    pub encounter_id: i32,
    /// The attempt index (1-based).
    pub attempt_index: i32,
    /// The timestamp of when the attempt started, in milliseconds since the Unix epoch.
    pub started_at_ms: i64,
    /// The timestamp of when the attempt ended, in milliseconds since the Unix epoch.
    pub ended_at_ms: Option<i64>,
    /// The reason for the attempt split ('wipe', 'hp_rollback', 'manual').
    pub reason: String,
    /// The boss HP at the start of the attempt.
    pub boss_hp_start: Option<i64>,
    /// The boss HP at the end of the attempt.
    pub boss_hp_end: Option<i64>,
    /// The total number of deaths in this attempt.
    pub total_deaths: i32,
}

/// Represents a row in the `encounter_phases` table.
#[derive(Debug, Clone, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(table_name = sch::encounter_phases, belongs_to(EncounterRow, foreign_key = encounter_id))]
pub struct EncounterPhaseRow {
    /// The unique ID of the encounter phase.
    pub id: i32,
    /// The ID of the encounter this phase belongs to.
    pub encounter_id: i32,
    /// The type of phase ('mob' or 'boss').
    pub phase_type: String,
    /// The timestamp of when the phase started, in milliseconds since the Unix epoch.
    pub start_time_ms: i64,
    /// The timestamp of when the phase ended, in milliseconds since the Unix epoch.
    pub end_time_ms: Option<i64>,
    /// The outcome of the phase ('success', 'wipe', 'unknown').
    pub outcome: String,
}

/// Represents a new encounter phase to be inserted into the `encounter_phases` table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sch::encounter_phases)]
pub struct NewEncounterPhase {
    /// The ID of the encounter this phase belongs to.
    pub encounter_id: i32,
    /// The type of phase ('mob' or 'boss').
    pub phase_type: String,
    /// The timestamp of when the phase started, in milliseconds since the Unix epoch.
    pub start_time_ms: i64,
    /// The timestamp of when the phase ended, in milliseconds since the Unix epoch.
    pub end_time_ms: Option<i64>,
    /// The outcome of the phase ('success', 'wipe', 'unknown').
    pub outcome: String,
}
