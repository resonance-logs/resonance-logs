use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::models as m;
use crate::database::schema as sch;
use crate::database::{default_db_path, ensure_parent_dir};
use crate::live::skill_names;
use crate::live::commands_models as lc;


/// A summary of an encounter.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSummaryDto {
    /// The ID of the encounter.
    pub id: i32,
    /// The start time of the encounter in milliseconds since the Unix epoch.
    pub started_at_ms: i64,
    /// The end time of the encounter in milliseconds since the Unix epoch.
    pub ended_at_ms: Option<i64>,
    /// The total damage dealt in the encounter.
    pub total_dmg: i64,
    /// The total healing done in the encounter.
    pub total_heal: i64,
    /// The ID of the scene where the encounter took place.
    pub scene_id: Option<i32>,
    /// The name of the scene where the encounter took place.
    pub scene_name: Option<String>,
    /// A list of bosses in the encounter.
    pub bosses: Vec<BossSummaryDto>,
    /// A list of players in the encounter.
    pub players: Vec<PlayerInfoDto>,
    /// A list of actor encounter stats.
    pub actors: Vec<ActorEncounterStatDto>,
}

/// The result of a query for recent encounters.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RecentEncountersResult {
    /// The rows of encounter summaries.
    pub rows: Vec<EncounterSummaryDto>,
    /// The total number of encounters.
    pub total_count: i64,
}

/// Filters for querying encounters.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterFiltersDto {
    /// A list of boss names to filter by.
    pub boss_names: Option<Vec<String>>,
    /// A list of encounter names to filter by.
    pub encounter_names: Option<Vec<String>>,
    /// A player name to filter by.
    pub player_name: Option<String>,
    /// A list of player names to filter by.
    pub player_names: Option<Vec<String>>,
    /// A list of class IDs to filter by.
    pub class_ids: Option<Vec<i32>>,
    /// The start date to filter by in milliseconds since the Unix epoch.
    pub date_from_ms: Option<i64>,
    /// The end date to filter by in milliseconds since the Unix epoch.
    pub date_to_ms: Option<i64>,
}

/// The result of a query for boss names.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BossNamesResult {
    /// A list of boss names.
    pub names: Vec<String>,
}

/// A summary of a boss.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BossSummaryDto {
    /// The name of the monster.
    pub monster_name: String,
    /// The maximum HP of the monster.
    pub max_hp: Option<i64>,
    /// Whether the boss was defeated.
    pub is_defeated: bool,
}

/// The result of a query for scene names.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SceneNamesResult {
    /// A list of scene names.
    pub names: Vec<String>,
}

/// The result of a query for player names.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PlayerNamesResult {
    /// A list of player names.
    pub names: Vec<String>,
}

/// An encounter with details.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterWithDetailsDto {
    /// The ID of the encounter.
    pub id: i32,
    /// The start time of the encounter in milliseconds since the Unix epoch.
    pub started_at_ms: i64,
    /// The end time of the encounter in milliseconds since the Unix epoch.
    pub ended_at_ms: Option<i64>,
    /// The total damage dealt in the encounter.
    pub total_dmg: i64,
    /// The total healing done in the encounter.
    pub total_heal: i64,
    /// The ID of the scene where the encounter took place.
    pub scene_id: Option<i32>,
    /// The name of the scene where the encounter took place.
    pub scene_name: Option<String>,
    /// A list of bosses in the encounter.
    pub bosses: Vec<BossSummaryDto>,
    /// A list of players in the encounter.
    pub players: Vec<PlayerInfoDto>,
}

/// Information about a player.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfoDto {
    /// The name of the player.
    pub name: String,
    /// The class ID of the player.
    pub class_id: Option<i32>,
    /// Whether the player is the local player.
    pub is_local_player: bool,
}

/// Statistics for an actor in an encounter.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ActorEncounterStatDto {
    /// The ID of the encounter.
    pub encounter_id: i32,
    /// The ID of the actor.
    pub actor_id: i64,
    /// The name of the actor.
    pub name: Option<String>,
    /// The class ID of the actor.
    pub class_id: Option<i32>,
    /// The ability score of the actor.
    pub ability_score: Option<i32>,
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
    /// Whether the actor is the local player.
    pub is_local_player: bool,
}

/// Loads the actor stats for a given encounter.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
/// * `encounter_id` - The ID of the encounter.
///
/// # Returns
///
/// * `Result<Vec<ActorEncounterStatDto>, String>` - A list of actor encounter stats.
fn load_actor_stats(
    conn: &mut diesel::sqlite::SqliteConnection,
    encounter_id: i32,
) -> Result<Vec<ActorEncounterStatDto>, String> {
    use sch::actor_encounter_stats::dsl as s;

    let rows = s::actor_encounter_stats
        .filter(s::encounter_id.eq(encounter_id))
        .filter(s::is_player.eq(1))
        .select((
            s::encounter_id,
            s::actor_id,
            s::name,
            s::class_id,
            s::ability_score,
            s::damage_dealt,
            s::heal_dealt,
            s::damage_taken,
            s::hits_dealt,
            s::hits_heal,
            s::hits_taken,
            s::crit_hits_dealt,
            s::crit_hits_heal,
            s::crit_hits_taken,
            s::lucky_hits_dealt,
            s::lucky_hits_heal,
            s::lucky_hits_taken,
            s::crit_total_dealt,
            s::crit_total_heal,
            s::crit_total_taken,
            s::lucky_total_dealt,
            s::lucky_total_heal,
            s::lucky_total_taken,
            s::boss_damage_dealt,
            s::boss_hits_dealt,
            s::boss_crit_hits_dealt,
            s::boss_lucky_hits_dealt,
            s::boss_crit_total_dealt,
            s::boss_lucky_total_dealt,
            s::is_local_player,
        ))
        .order((
            s::damage_dealt.desc(),
            s::heal_dealt.desc(),
            s::damage_taken.desc(),
        ))
        .load::<(
            i32,
            i64,
            Option<String>,
            Option<i32>,
            Option<i32>,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i32,
        )>(conn)
        .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(
            |(
                encounter_id,
                actor_id,
                name,
                class_id,
                ability_score,
                damage_dealt,
                heal_dealt,
                damage_taken,
                hits_dealt,
                hits_heal,
                hits_taken,
                crit_hits_dealt,
                crit_hits_heal,
                crit_hits_taken,
                lucky_hits_dealt,
                lucky_hits_heal,
                lucky_hits_taken,
                crit_total_dealt,
                crit_total_heal,
                crit_total_taken,
                lucky_total_dealt,
                lucky_total_heal,
                lucky_total_taken,
                boss_damage_dealt,
                boss_hits_dealt,
                boss_crit_hits_dealt,
                boss_lucky_hits_dealt,
                boss_crit_total_dealt,
                boss_lucky_total_dealt,
                is_local_player,
            )| ActorEncounterStatDto {
                encounter_id,
                actor_id,
                name,
                class_id,
                ability_score,
                damage_dealt,
                heal_dealt,
                damage_taken,
                hits_dealt,
                hits_heal,
                hits_taken,
                crit_hits_dealt,
                crit_hits_heal,
                crit_hits_taken,
                lucky_hits_dealt,
                lucky_hits_heal,
                lucky_hits_taken,
                crit_total_dealt,
                crit_total_heal,
                crit_total_taken,
                lucky_total_dealt,
                lucky_total_heal,
                lucky_total_taken,
                boss_damage_dealt,
                boss_hits_dealt,
                boss_crit_hits_dealt,
                boss_lucky_hits_dealt,
                boss_crit_total_dealt,
                boss_lucky_total_dealt,
                is_local_player: is_local_player == 1,
            },
        )
        .collect())
}

/// Gets a database connection.
///
/// # Returns
///
/// * `Result<diesel::sqlite::SqliteConnection, String>` - A database connection.
fn get_conn() -> Result<diesel::sqlite::SqliteConnection, String> {
    let path = default_db_path();
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy()).map_err(|e| e.to_string())
}

/// Gets a list of unique boss names.
///
/// # Returns
///
/// * `Result<BossNamesResult, String>` - A list of unique boss names.
#[tauri::command]
#[specta::specta]
pub fn get_unique_boss_names() -> Result<BossNamesResult, String> {
    let mut conn = get_conn()?;
    use sch::encounter_bosses::dsl as eb;
    use std::collections::HashSet;

    // Use the materialized encounter_bosses table (damage_events is deprecated)
    let boss_names: Vec<String> = eb::encounter_bosses
        .select(eb::monster_name)
        .load::<String>(&mut conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    Ok(BossNamesResult { names: boss_names })
}

/// Gets a list of unique scene names.
///
/// # Returns
///
/// * `Result<SceneNamesResult, String>` - A list of unique scene names.
#[tauri::command]
#[specta::specta]
pub fn get_unique_scene_names() -> Result<SceneNamesResult, String> {
    let mut conn = get_conn()?;
    use std::collections::HashSet;

    let scene_names: Vec<Option<String>> = sch::encounters::dsl::encounters
        .select(sch::encounters::dsl::scene_name)
        .load::<Option<String>>(&mut conn)
        .map_err(|e| e.to_string())?;

    let names: Vec<String> = scene_names
        .into_iter()
        .flatten()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    Ok(SceneNamesResult { names })
}

/// Gets a list of player names filtered by a prefix.
///
/// # Arguments
///
/// * `prefix` - The prefix to filter by.
///
/// # Returns
///
/// * `Result<PlayerNamesResult, String>` - A list of player names.
#[tauri::command]
#[specta::specta]
pub fn get_player_names_filtered(prefix: String) -> Result<PlayerNamesResult, String> {
    // Only query if prefix is at least 3 characters
    if prefix.trim().len() < 3 {
        return Ok(PlayerNamesResult { names: vec![] });
    }

    let mut conn = get_conn()?;
    use sch::actor_encounter_stats::dsl as s;
    use std::collections::HashSet;

    let pattern = format!("%{}%", prefix.trim());
    let player_names: Vec<String> = s::actor_encounter_stats
        .select(s::name)
        .filter(s::is_player.eq(1))
        .filter(s::name.is_not_null())
        .filter(s::name.like(pattern))
        .load::<Option<String>>(&mut conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .flatten()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    Ok(PlayerNamesResult { names: player_names })
}

/// Gets a list of recent encounters filtered by the given criteria.
///
/// # Arguments
///
/// * `limit` - The maximum number of encounters to return.
/// * `offset` - The number of encounters to skip.
/// * `filters` - The filters to apply.
///
/// # Returns
///
/// * `Result<RecentEncountersResult, String>` - A list of recent encounters.
#[tauri::command]
#[specta::specta]
pub fn get_recent_encounters_filtered(
    limit: i32,
    offset: i32,
    filters: Option<EncounterFiltersDto>,
) -> Result<RecentEncountersResult, String> {
    let mut conn = get_conn()?;
    use sch::encounters::dsl as e;
    use sch::encounter_bosses::dsl as eb;
    use sch::actor_encounter_stats::dsl as s;
    use std::collections::HashSet;

    // Start with base query for encounters
    let mut encounter_query = e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .into_boxed();
    let mut count_query = e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .into_boxed();

    let mut encounter_id_filters: Option<HashSet<i32>> = None;

    if let Some(ref filter) = filters {
        if let Some(ref boss_names) = filter.boss_names {
            if !boss_names.is_empty() {
                let boss_ids: HashSet<i32> = eb::encounter_bosses
                    .filter(eb::monster_name.eq_any(boss_names))
                    .select(eb::encounter_id)
                    .load::<i32>(&mut conn)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .collect();

                if let Some(existing) = &mut encounter_id_filters {
                    existing.retain(|id| boss_ids.contains(id));
                } else {
                    encounter_id_filters = Some(boss_ids);
                }
            }
        }

        if let Some(ref encounter_names) = filter.encounter_names {
            if !encounter_names.is_empty() {
                // Find encounters that have matching scene names
                let scene_encounter_ids: HashSet<i32> = e::encounters
                    .filter(e::scene_name.eq_any(encounter_names))
                    .select(e::id)
                    .load::<i32>(&mut conn)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .collect();

                if let Some(existing) = &mut encounter_id_filters {
                    existing.retain(|id| scene_encounter_ids.contains(id));
                } else {
                    encounter_id_filters = Some(scene_encounter_ids);
                }
            }
        }

        if let Some(ref player_names) = filter.player_names {
            if !player_names.is_empty() {
                let player_encounter_ids: HashSet<i32> = s::actor_encounter_stats
                    .filter(s::is_player.eq(1))
                    .filter(s::name.eq_any(player_names))
                    .select(s::encounter_id)
                    .load::<i32>(&mut conn)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .collect();

                if let Some(existing) = &mut encounter_id_filters {
                    existing.retain(|id| player_encounter_ids.contains(id));
                } else {
                    encounter_id_filters = Some(player_encounter_ids);
                }
            }
        }

        if let Some(ref player_name) = filter.player_name {
            let trimmed = player_name.trim();
            if !trimmed.is_empty() {
                let pattern = format!("%{}%", trimmed);
                let player_ids: HashSet<i32> = s::actor_encounter_stats
                    .filter(s::is_player.eq(1))
                    .filter(s::name.is_not_null())
                    .filter(s::name.like(pattern))
                    .select(s::encounter_id)
                    .load::<i32>(&mut conn)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .collect();

                if let Some(existing) = &mut encounter_id_filters {
                    existing.retain(|id| player_ids.contains(id));
                } else {
                    encounter_id_filters = Some(player_ids);
                }
            }
        }

        if let Some(ref class_ids) = filter.class_ids {
            if !class_ids.is_empty() {
                let class_filters: Vec<Option<i32>> = class_ids.iter().map(|id| Some(*id)).collect();
                let class_encounter_ids: HashSet<i32> = s::actor_encounter_stats
                    .filter(s::is_player.eq(1))
                    .filter(s::class_id.is_not_null())
                    .filter(s::class_id.eq_any(class_filters))
                    .select(s::encounter_id)
                    .load::<i32>(&mut conn)
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .collect();

                if let Some(existing) = &mut encounter_id_filters {
                    existing.retain(|id| class_encounter_ids.contains(id));
                } else {
                    encounter_id_filters = Some(class_encounter_ids);
                }
            }
        }

        if let Some(date_from_ms) = filter.date_from_ms {
            encounter_query = encounter_query.filter(e::started_at_ms.ge(date_from_ms));
            count_query = count_query.filter(e::started_at_ms.ge(date_from_ms));
        }

        if let Some(date_to_ms) = filter.date_to_ms {
            encounter_query = encounter_query.filter(e::started_at_ms.le(date_to_ms));
            count_query = count_query.filter(e::started_at_ms.le(date_to_ms));
        }
    }

    let encounter_id_vec = encounter_id_filters.map(|set| set.into_iter().collect::<Vec<_>>());

    if let Some(ref ids) = encounter_id_vec {
        if ids.is_empty() {
            return Ok(RecentEncountersResult {
                rows: Vec::new(),
                total_count: 0,
            });
        }
        encounter_query = encounter_query.filter(e::id.eq_any(ids.clone()));
        count_query = count_query.filter(e::id.eq_any(ids.clone()));
    }

    // Get encounter rows
    let rows: Vec<(i32, i64, Option<i64>, Option<i64>, Option<i64>, Option<i32>, Option<String>)> = encounter_query
        .order(e::started_at_ms.desc())
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
            e::scene_id,
            e::scene_name,
        ))
        .limit(limit as i64)
        .offset(offset as i64)
        .load(&mut conn)
        .map_err(|er| er.to_string())?;

    // Get total count for pagination
    let total_count: i64 = count_query
        .count()
        .get_result(&mut conn)
        .map_err(|er| er.to_string())?;

    // Collect boss and player data for each encounter
    let mut mapped: Vec<EncounterSummaryDto> = Vec::new();

    for (id, started, ended, td, th, scene_id, scene_name) in rows {
        // Get unique boss entries (name + max_hp) from the materialized encounter_bosses
        let boss_rows: Vec<(String, Option<i64>, i32)> = eb::encounter_bosses
            .filter(eb::encounter_id.eq(id))
            .select((eb::monster_name, eb::max_hp, eb::is_defeated))
            .load::<(String, Option<i64>, i32)>(&mut conn)
            .map_err(|er| er.to_string())?;

        use std::collections::HashSet as StdHashSet;
        let boss_names_set: StdHashSet<(String, Option<i64>, i32)> = boss_rows.into_iter().collect();
        let boss_entries: Vec<BossSummaryDto> = boss_names_set
            .into_iter()
            .map(|(name, max_hp, defeated)| BossSummaryDto { monster_name: name, max_hp, is_defeated: defeated != 0 })
            .collect();

        // Get unique player names and class_ids from actor_encounter_stats where is_player=1
        let player_data: Vec<PlayerInfoDto> = s::actor_encounter_stats
            .filter(s::encounter_id.eq(id))
            .filter(s::is_player.eq(1))
            .select((s::name, s::class_id, s::is_local_player))
            .load::<(Option<String>, Option<i32>, i32)>(&mut conn)
            .map_err(|er| er.to_string())?
            .into_iter()
            .filter_map(|(name, class_id, is_local)| name.map(|n| PlayerInfoDto {
                name: n,
                class_id,
                is_local_player: is_local != 0,
            }))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        mapped.push(EncounterSummaryDto {
            id,
            started_at_ms: started,
            ended_at_ms: ended,
            total_dmg: td.unwrap_or(0),
            total_heal: th.unwrap_or(0),
            scene_id,
            scene_name,
            bosses: boss_entries,
            players: player_data,
            actors: Vec::new(),
        });
    }

    Ok(RecentEncountersResult {
        rows: mapped,
        total_count,
    })
}

/// Gets a list of recent encounters.
///
/// # Arguments
///
/// * `limit` - The maximum number of encounters to return.
/// * `offset` - The number of encounters to skip.
///
/// # Returns
///
/// * `Result<RecentEncountersResult, String>` - A list of recent encounters.
#[tauri::command]
#[specta::specta]
pub fn get_recent_encounters(limit: i32, offset: i32) -> Result<RecentEncountersResult, String> {
    let mut conn = get_conn()?;
    use sch::encounters::dsl as e;
    let rows: Vec<(i32, i64, Option<i64>, Option<i64>, Option<i64>, Option<i32>, Option<String>)> = e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .order(e::started_at_ms.desc())
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
            e::scene_id,
            e::scene_name,
        ))
        .limit(limit as i64)
        .offset(offset as i64)
        .load(&mut conn)
        .map_err(|er| er.to_string())?;

    let total_count: i64 = e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .count()
        .get_result(&mut conn)
        .map_err(|er| er.to_string())?;

    // Collect boss and player data for each encounter
    let mut mapped: Vec<EncounterSummaryDto> = Vec::new();

    for (id, started, ended, td, th, scene_id, scene_name) in rows {
        use sch::actor_encounter_stats::dsl as s;
        use sch::encounter_bosses::dsl as eb;
        use std::collections::HashSet;

        // Get unique boss entries (name + max_hp + defeated) from the materialized encounter_bosses
        let boss_rows: Vec<(String, Option<i64>, i32)> = eb::encounter_bosses
            .filter(eb::encounter_id.eq(id))
            .select((eb::monster_name, eb::max_hp, eb::is_defeated))
            .load::<(String, Option<i64>, i32)>(&mut conn)
            .map_err(|er| er.to_string())?;

        use std::collections::HashSet as StdHashSet;
        let boss_names_set: StdHashSet<(String, Option<i64>, i32)> = boss_rows.into_iter().collect();
        let boss_entries: Vec<BossSummaryDto> = boss_names_set
            .into_iter()
            .map(|(name, max_hp, defeated)| BossSummaryDto { monster_name: name, max_hp, is_defeated: defeated != 0 })
            .collect();

        // Get unique player names and class_ids from actor_encounter_stats where is_player=1
        let player_data: Vec<PlayerInfoDto> = s::actor_encounter_stats
            .filter(s::encounter_id.eq(id))
            .filter(s::is_player.eq(1))
            .select((s::name, s::class_id, s::is_local_player))
            .load::<(Option<String>, Option<i32>, i32)>(&mut conn)
            .map_err(|er| er.to_string())?
            .into_iter()
            .filter_map(|(name, class_id, is_local)| name.map(|n| PlayerInfoDto {
                name: n,
                class_id,
                is_local_player: is_local != 0,
            }))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        mapped.push(EncounterSummaryDto {
            id,
            started_at_ms: started,
            ended_at_ms: ended,
            total_dmg: td.unwrap_or(0),
            total_heal: th.unwrap_or(0),
            scene_id,
            scene_name,
            bosses: boss_entries,
            players: player_data,
            actors: Vec::new(),
        });
    }

    Ok(RecentEncountersResult {
        rows: mapped,
        total_count,
    })
}

/// Gets the actor stats for a given encounter.
///
/// # Arguments
///
/// * `encounter_id` - The ID of the encounter.
///
/// # Returns
///
/// * `Result<Vec<ActorEncounterStatDto>, String>` - A list of actor encounter stats.
#[tauri::command]
#[specta::specta]
pub fn get_encounter_actor_stats(encounter_id: i32) -> Result<Vec<ActorEncounterStatDto>, String> {
    let mut conn = get_conn()?;
    load_actor_stats(&mut conn, encounter_id)
}

/// Get player name by UID from database
///
/// # Arguments
///
/// * `uid` - The UID of the player.
///
/// # Returns
///
/// * `Result<Option<String>, String>` - The name of the player, or `None` if not found.
pub fn get_name_by_uid(uid: i64) -> Result<Option<String>, String> {
    let mut conn = get_conn()?;
    use sch::entities::dsl as en;

    let name: Option<Option<String>> = en::entities
        .select(en::name)
        .filter(en::entity_id.eq(uid))
        .first::<Option<String>>(&mut conn)
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(name.flatten())
}

/// Get recent players ordered by last_seen_ms (most recent first) kinda scuffed maybe update in future
///
/// # Arguments
///
/// * `limit` - The maximum number of players to return.
///
/// # Returns
///
/// * `Result<Vec<(i64, String)>, String>` - A list of recent players.
pub fn get_recent_players(limit: i64) -> Result<Vec<(i64, String)>, String> {
    let mut conn = get_conn()?;
    use sch::entities::dsl as en;

    let rows: Vec<(i64, Option<String>)> = en::entities
        .select((en::entity_id, en::name))
        .filter(en::name.is_not_null())
        .order(en::last_seen_ms.desc())
        .limit(limit)
        .load(&mut conn)
        .map_err(|e: diesel::result::Error| e.to_string())?;

    Ok(rows
        .into_iter()
        .filter_map(|(uid, name_opt)| name_opt.map(|name| (uid, name)))
        .collect())
}

/// A Tauri command to get a list of recent players.
///
/// # Arguments
///
/// * `limit` - The maximum number of players to return.
///
/// # Returns
///
/// * `Result<Vec<(i64, String)>, String>` - A list of recent players.
#[tauri::command]
#[specta::specta]
pub fn get_recent_players_command(limit: i32) -> Result<Vec<(i64, String)>, String> {
    get_recent_players(limit as i64)
}

/// A Tauri command to get the name of a player by their UID.
///
/// # Arguments
///
/// * `uid` - The UID of the player.
///
/// # Returns
///
/// * `Result<Option<String>, String>` - The name of the player, or `None` if not found.
#[tauri::command]
#[specta::specta]
pub fn get_player_name_command(uid: i64) -> Result<Option<String>, String> {
    get_name_by_uid(uid)
}

/// Gets an encounter by its ID.
///
/// # Arguments
///
/// * `encounter_id` - The ID of the encounter.
///
/// # Returns
///
/// * `Result<EncounterSummaryDto, String>` - The encounter summary.
#[tauri::command]
#[specta::specta]
pub fn get_encounter_by_id(encounter_id: i32) -> Result<EncounterSummaryDto, String> {
    let mut conn = get_conn()?;
    use sch::encounter_bosses::dsl as eb;
    use sch::encounters::dsl as e;
    use std::collections::HashSet;

    let row: (i32, i64, Option<i64>, Option<i64>, Option<i64>, Option<i32>, Option<String>) = e::encounters
        .filter(e::id.eq(encounter_id))
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
            e::scene_id,
            e::scene_name,
        ))
        .first(&mut conn)
        .map_err(|er| er.to_string())?;

    let actors = load_actor_stats(&mut conn, encounter_id)?;

    let boss_rows: Vec<(String, Option<i64>, i32)> = eb::encounter_bosses
        .filter(eb::encounter_id.eq(encounter_id))
        .select((eb::monster_name, eb::max_hp, eb::is_defeated))
        .load::<(String, Option<i64>, i32)>(&mut conn)
        .map_err(|er| er.to_string())?;

    use std::collections::HashSet as StdHashSet;
    let boss_set: StdHashSet<(String, Option<i64>, i32)> = boss_rows.into_iter().collect();
    let boss_names: Vec<BossSummaryDto> = boss_set
        .into_iter()
        .map(|(name, max_hp, defeated)| BossSummaryDto { monster_name: name, max_hp, is_defeated: defeated != 0 })
        .collect();

    let players: Vec<PlayerInfoDto> = actors
        .iter()
        .filter_map(|actor| {
            actor.name.as_ref().map(|name| PlayerInfoDto {
                name: name.clone(),
                class_id: actor.class_id,
                is_local_player: actor.is_local_player,
            })
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    Ok(EncounterSummaryDto {
        id: row.0,
        started_at_ms: row.1,
        ended_at_ms: row.2,
        total_dmg: row.3.unwrap_or(0),
        total_heal: row.4.unwrap_or(0),
        scene_id: row.5,
        scene_name: row.6.clone(),
        bosses: boss_names,
        players,
        actors,
    })
}

/// Deletes an encounter by its ID.
///
/// # Arguments
///
/// * `encounter_id` - The ID of the encounter to delete.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result indicating success or failure.
#[tauri::command]
#[specta::specta]
pub fn delete_encounter(encounter_id: i32) -> Result<(), String> {
    let mut conn = get_conn()?;
    use sch::actor_encounter_stats::dsl as s;
    use sch::damage_events::dsl as de;
    use sch::encounters::dsl as e;
    use sch::heal_events::dsl as he;

    conn.transaction::<(), diesel::result::Error, _>(|conn| {
        diesel::delete(de::damage_events.filter(de::encounter_id.eq(encounter_id)))
            .execute(conn)?;
        diesel::delete(he::heal_events.filter(he::encounter_id.eq(encounter_id))).execute(conn)?;
        diesel::delete(s::actor_encounter_stats.filter(s::encounter_id.eq(encounter_id)))
            .execute(conn)?;
        diesel::delete(e::encounters.filter(e::id.eq(encounter_id))).execute(conn)?;
        Ok(())
    })
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Gets a list of recent encounters with details.
///
/// # Arguments
///
/// * `limit` - The maximum number of encounters to return.
/// * `offset` - The number of encounters to skip.
///
/// # Returns
///
/// * `Result<Vec<EncounterWithDetailsDto>, String>` - A list of recent encounters with details.
#[tauri::command]
#[specta::specta]
pub fn get_recent_encounters_with_details(
    limit: i32,
    offset: i32,
) -> Result<Vec<EncounterWithDetailsDto>, String> {
    let mut conn = get_conn()?;
    use sch::actor_encounter_stats::dsl as s;
    use sch::encounter_bosses::dsl as eb;
    use sch::encounters::dsl as e;
    use std::collections::HashSet;

    // Get encounter summaries
    let encounter_rows: Vec<(i32, i64, Option<i64>, Option<i64>, Option<i64>, Option<i32>, Option<String>)> = e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .order(e::started_at_ms.desc())
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
            e::scene_id,
            e::scene_name,
        ))
        .limit(limit as i64)
        .offset(offset as i64)
        .load(&mut conn)
        .map_err(|er| er.to_string())?;

    let mut results = Vec::new();

    for (id, started_at_ms, ended_at_ms, total_dmg, total_heal, scene_id, scene_name) in encounter_rows {
        // Get unique boss entries (name + max_hp + defeated) from the materialized encounter_bosses
        let boss_rows: Vec<(String, Option<i64>, i32)> = eb::encounter_bosses
            .filter(eb::encounter_id.eq(id))
            .select((eb::monster_name, eb::max_hp, eb::is_defeated))
            .load::<(String, Option<i64>, i32)>(&mut conn)
            .map_err(|er| er.to_string())?;

        use std::collections::HashSet as StdHashSet;
        let boss_names_set: StdHashSet<(String, Option<i64>, i32)> = boss_rows.into_iter().collect();
        let boss_entries: Vec<BossSummaryDto> = boss_names_set
            .into_iter()
            .map(|(name, max_hp, defeated)| BossSummaryDto { monster_name: name, max_hp, is_defeated: defeated != 0 })
            .collect();

        // Get unique player names and class_ids from actor_encounter_stats where is_player=1
        let player_data: Vec<PlayerInfoDto> = s::actor_encounter_stats
            .filter(s::encounter_id.eq(id))
            .filter(s::is_player.eq(1))
            .select((s::name, s::class_id, s::is_local_player))
            .load::<(Option<String>, Option<i32>, i32)>(&mut conn)
            .map_err(|er| er.to_string())?
            .into_iter()
            .filter_map(|(name, class_id, is_local)| name.map(|n| PlayerInfoDto {
                name: n,
                class_id,
                is_local_player: is_local != 0,
            }))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        results.push(EncounterWithDetailsDto {
            id,
            started_at_ms,
            ended_at_ms,
            total_dmg: total_dmg.unwrap_or(0),
            total_heal: total_heal.unwrap_or(0),
            scene_id,
            scene_name,
            bosses: boss_entries,
            players: player_data,
        });
    }

    Ok(results)
}

/// Gets the skills used by a player in an encounter.
///
/// # Arguments
///
/// * `encounter_id` - The ID of the encounter.
/// * `actor_id` - The ID of the actor.
/// * `skill_type` - The type of skill to get (e.g., "dps", "heal").
///
/// # Returns
///
/// * `Result<lc::SkillsWindow, String>` - The skills window.
#[tauri::command]
#[specta::specta]
pub fn get_encounter_player_skills(
    encounter_id: i32,
    actor_id: i64,
    skill_type: String,
) -> Result<lc::SkillsWindow, String> {
    let mut conn = get_conn()?;
    use sch::actor_encounter_stats::dsl as s;
    use sch::damage_skill_stats::dsl as dss;
    use sch::encounters::dsl as e;
    use sch::heal_skill_stats::dsl as hss;
    use std::collections::HashMap;

    // Get encounter timings
    let encounter_row: (i32, i64, Option<i64>, Option<i64>, Option<i64>, Option<i32>, Option<String>) = e::encounters
        .filter(e::id.eq(encounter_id))
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
            e::scene_id,
            e::scene_name,
        ))
        .first(&mut conn)
        .map_err(|er| er.to_string())?;

    let started = encounter_row.1;
    let ended_opt = encounter_row.2;
    let duration_secs = match ended_opt {
        Some(ended) if ended > started => ((ended - started) as f64) / 1000.0,
        _ => 1.0,
    };

    // Get actor totals from actor_encounter_stats
    let actor_row_opt = s::actor_encounter_stats
        .filter(s::encounter_id.eq(encounter_id))
        .filter(s::actor_id.eq(actor_id))
        .select((
            s::damage_dealt,
            s::heal_dealt,
            s::hits_dealt,
            s::crit_hits_dealt,
            s::lucky_hits_dealt,
        ))
        .first::<(i64, i64, i64, i64, i64)>(&mut conn)
        .optional()
        .map_err(|e| e.to_string())?;

    let (actor_total_dmg, actor_total_heal, actor_hits, actor_crit_hits, actor_lucky_hits) =
        actor_row_opt.unwrap_or((0, 0, 0, 0, 0));

    // Build curr_player PlayerRow (use minimal fields similar to live PlayerRow)
    // Attempt to get name and ability_score from entities
    use sch::entities::dsl as en;
    let (name_opt, ability_score_opt) = en::entities
        .filter(en::entity_id.eq(actor_id))
        .select((en::name.nullable(), en::ability_score.nullable()))
        .first::<(Option<String>, Option<i32>)>(&mut conn)
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or((None, None));

    let player_name = name_opt.unwrap_or_else(|| String::from(""));

    let curr_player = lc::PlayerRow {
        uid: actor_id as u128,
        name: player_name.clone(),
        class_name: String::from(""),
        class_spec_name: String::from(""),
        ability_score: ability_score_opt.unwrap_or(0) as u128,
        total_dmg: actor_total_dmg as u128,
        dps: if duration_secs > 0.0 {
            (actor_total_dmg as f64) / duration_secs
        } else {
            0.0
        },
        dmg_pct: 0.0, // filled per-skill below if needed
        crit_rate: if actor_hits > 0 {
            (actor_crit_hits as f64) / (actor_hits as f64)
        } else {
            0.0
        },
        crit_dmg_rate: 0.0,
        lucky_rate: if actor_hits > 0 {
            (actor_lucky_hits as f64) / (actor_hits as f64)
        } else {
            0.0
        },
        lucky_dmg_rate: 0.0,
        hits: actor_hits as u128,
        hits_per_minute: if duration_secs > 0.0 {
            (actor_hits as f64) / (duration_secs / 60.0)
        } else {
            0.0
        },
        // Extended attributes from Stage 4 (not available in historical data)
        rank_level: None,
        current_hp: None,
        max_hp: None,
        crit_stat: None,
        lucky_stat: None,
        haste: None,
        mastery: None,
        element_flag: None,
        energy_flag: None,
        reduction_level: None,
    };

    // Aggregate skills depending on skill_type
    let mut skill_rows: Vec<lc::SkillRow> = Vec::new();

    if (skill_type == "dps" || skill_type == "tanked") {
        let stats = dss::damage_skill_stats
            .filter(dss::encounter_id.eq(encounter_id))
            .filter(dss::attacker_id.eq(actor_id))
            .load::<m::DamageSkillStatRow>(&mut conn)
            .map_err(|e| e.to_string())?;

        let mut agg: HashMap<i32, (i64, i64, i64, i64, i64, i64)> = HashMap::new();
        for stat in stats {
            let entry = agg
                .entry(stat.skill_id)
                .or_insert((0, 0, 0, 0, 0, 0));
            entry.0 += stat.total_value;
            entry.1 += stat.hits as i64;
            entry.2 += stat.crit_hits as i64;
            entry.3 += stat.lucky_hits as i64;
            entry.4 += stat.crit_total;
            entry.5 += stat.lucky_total;
        }

        let mut items: Vec<(i32, (i64, i64, i64, i64, i64, i64))> = agg.into_iter().collect();
        items.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

        for (skill_id, (total_dmg, hits, crit_hits, lucky_hits, crit_total, lucky_total)) in items {
            let name = if skill_id > 0 {
                skill_names::lookup(skill_id)
                    .unwrap_or_else(|| String::from("Unknown Skill"))
            } else {
                String::from("Unknown Skill")
            };

            let hits_f = hits as f64;
            let total_dmg_f = total_dmg as f64;
            let sr = lc::SkillRow {
                name,
                total_dmg: total_dmg.max(0) as u128,
                dps: if duration_secs > 0.0 {
                    total_dmg_f / duration_secs
                } else {
                    0.0
                },
                dmg_pct: if actor_total_dmg > 0 {
                    total_dmg_f * 100.0 / (actor_total_dmg as f64)
                } else {
                    0.0
                },
                crit_rate: if hits > 0 {
                    (crit_hits as f64) / hits_f
                } else {
                    0.0
                },
                crit_dmg_rate: if total_dmg > 0 {
                    (crit_total as f64) / total_dmg_f
                } else {
                    0.0
                },
                lucky_rate: if hits > 0 {
                    (lucky_hits as f64) / hits_f
                } else {
                    0.0
                },
                lucky_dmg_rate: if total_dmg > 0 {
                    (lucky_total as f64) / total_dmg_f
                } else {
                    0.0
                },
                hits: hits.max(0) as u128,
                hits_per_minute: if duration_secs > 0.0 {
                    hits_f / (duration_secs / 60.0)
                } else {
                    0.0
                },
            };
            skill_rows.push(sr);
        }
    } else if skill_type == "heal" {
        let stats = hss::heal_skill_stats
            .filter(hss::encounter_id.eq(encounter_id))
            .filter(hss::healer_id.eq(actor_id))
            .load::<m::HealSkillStatRow>(&mut conn)
            .map_err(|e| e.to_string())?;

        let mut agg: HashMap<i32, (i64, i64, i64, i64, i64, i64)> = HashMap::new();
        for stat in stats {
            let entry = agg
                .entry(stat.skill_id)
                .or_insert((0, 0, 0, 0, 0, 0));
            entry.0 += stat.total_value;
            entry.1 += stat.hits as i64;
            entry.2 += stat.crit_hits as i64;
            entry.3 += stat.lucky_hits as i64;
            entry.4 += stat.crit_total;
            entry.5 += stat.lucky_total;
        }

        let mut items: Vec<(i32, (i64, i64, i64, i64, i64, i64))> = agg.into_iter().collect();
        items.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

        for (skill_id, (total_heal, hits, crit_hits, lucky_hits, crit_total, lucky_total)) in items {
            let name = if skill_id > 0 {
                skill_names::lookup(skill_id)
                    .unwrap_or_else(|| String::from("Unknown Skill"))
            } else {
                String::from("Unknown Skill")
            };

            let hits_f = hits as f64;
            let total_heal_f = total_heal as f64;
            let sr = lc::SkillRow {
                name,
                total_dmg: total_heal.max(0) as u128,
                dps: if duration_secs > 0.0 {
                    total_heal_f / duration_secs
                } else {
                    0.0
                },
                dmg_pct: if actor_total_heal > 0 {
                    total_heal_f * 100.0 / (actor_total_heal as f64)
                } else {
                    0.0
                },
                crit_rate: if hits > 0 {
                    (crit_hits as f64) / hits_f
                } else {
                    0.0
                },
                crit_dmg_rate: if total_heal > 0 {
                    (crit_total as f64) / total_heal_f
                } else {
                    0.0
                },
                lucky_rate: if hits > 0 {
                    (lucky_hits as f64) / hits_f
                } else {
                    0.0
                },
                lucky_dmg_rate: if total_heal > 0 {
                    (lucky_total as f64) / total_heal_f
                } else {
                    0.0
                },
                hits: hits.max(0) as u128,
                hits_per_minute: if duration_secs > 0.0 {
                    hits_f / (duration_secs / 60.0)
                } else {
                    0.0
                },
            };
            skill_rows.push(sr);
        }
    } else {
        return Err(format!("Invalid skill type: {}", skill_type));
    }

    let sw = lc::SkillsWindow {
        curr_player: vec![curr_player],
        skill_rows,
    };

    Ok(sw)
}
