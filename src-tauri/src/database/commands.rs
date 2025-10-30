use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::models as m;
use crate::database::schema as sch;
use crate::database::{default_db_path, ensure_parent_dir};
use crate::live::skill_names;
use crate::live::commands_models as lc;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSummaryDto {
    pub id: i32,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub total_dmg: i64,
    pub total_heal: i64,
    pub bosses: Vec<String>,
    pub players: Vec<PlayerInfoDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RecentEncountersResult {
    pub rows: Vec<EncounterSummaryDto>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BossFilterDto {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BossNamesResult {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterWithDetailsDto {
    pub id: i32,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub total_dmg: i64,
    pub total_heal: i64,
    pub bosses: Vec<String>,
    pub players: Vec<PlayerInfoDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfoDto {
    pub name: String,
    pub class_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ActorEncounterStatDto {
    pub encounter_id: i32,
    pub actor_id: i64,
    pub name: Option<String>,
    pub class_id: Option<i32>,
    pub damage_dealt: i64,
    pub heal_dealt: i64,
    pub damage_taken: i64,
    pub hits_dealt: i64,
    pub hits_heal: i64,
    pub hits_taken: i64,
    pub crit_hits_dealt: i64,
    pub crit_hits_heal: i64,
    pub crit_hits_taken: i64,
    pub lucky_hits_dealt: i64,
    pub lucky_hits_heal: i64,
    pub lucky_hits_taken: i64,
    pub boss_damage_dealt: i64,
    pub boss_hits_dealt: i64,
    pub boss_crit_hits_dealt: i64,
    pub boss_lucky_hits_dealt: i64,
    pub boss_crit_total_dealt: i64,
    pub boss_lucky_total_dealt: i64,
}

fn get_conn() -> Result<diesel::sqlite::SqliteConnection, String> {
    let path = default_db_path();
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy()).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_unique_boss_names() -> Result<BossNamesResult, String> {
    let mut conn = get_conn()?;
    use sch::damage_events::dsl as de;
    use std::collections::HashSet;

    let boss_names: Vec<String> = de::damage_events
        .filter(de::is_boss.eq(1))
        .filter(de::monster_name.is_not_null())
        .select(de::monster_name)
        .load::<Option<String>>(&mut conn)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter_map(|n| n)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    Ok(BossNamesResult { names: boss_names })
}

#[tauri::command]
#[specta::specta]
pub fn get_recent_encounters_filtered(
    limit: i32,
    offset: i32,
    boss_filter: Option<BossFilterDto>,
) -> Result<RecentEncountersResult, String> {
    let mut conn = get_conn()?;
    use sch::encounters::dsl as e;
    use sch::damage_events::dsl as de;
    use sch::actor_encounter_stats::dsl as s;
    use std::collections::HashSet;

    // Start with base query for encounters
    let mut encounter_query = e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .into_boxed();

    // If boss filter is provided, we need to filter encounters that contain any of the specified bosses
    if let Some(ref filter) = boss_filter {
        if !filter.names.is_empty() {
            // Get encounter IDs that have any of the specified bosses
            let filtered_encounter_ids: Vec<i32> = de::damage_events
                .filter(de::is_boss.eq(1))
                .filter(de::monster_name.eq_any(&filter.names))
                .select(de::encounter_id)
                .load::<i32>(&mut conn)
                .map_err(|e| e.to_string())?
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            encounter_query = encounter_query.filter(e::id.eq_any(filtered_encounter_ids));
        }
    }

    // Get encounter rows
    let rows: Vec<(i32, i64, Option<i64>, Option<i64>, Option<i64>)> = encounter_query
        .order(e::started_at_ms.desc())
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
        ))
        .limit(limit as i64)
        .offset(offset as i64)
        .load(&mut conn)
        .map_err(|er| er.to_string())?;

    // Get total count for pagination
    let total_count: i64 = if let Some(ref filter) = boss_filter {
        if !filter.names.is_empty() {
            let filtered_encounter_ids: Vec<i32> = de::damage_events
                .filter(de::is_boss.eq(1))
                .filter(de::monster_name.eq_any(&filter.names))
                .select(de::encounter_id)
                .load::<i32>(&mut conn)
                .map_err(|e| e.to_string())?
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            e::encounters
                .filter(e::ended_at_ms.is_not_null())
                .filter(e::id.eq_any(filtered_encounter_ids))
                .count()
                .get_result(&mut conn)
                .map_err(|er| er.to_string())?
        } else {
            e::encounters
                .filter(e::ended_at_ms.is_not_null())
                .count()
                .get_result(&mut conn)
                .map_err(|er| er.to_string())?
        }
    } else {
        e::encounters
            .filter(e::ended_at_ms.is_not_null())
            .count()
            .get_result(&mut conn)
            .map_err(|er| er.to_string())?
    };

    // Collect boss and player data for each encounter
    let mut mapped: Vec<EncounterSummaryDto> = Vec::new();

    for (id, started, ended, td, th) in rows {
        // Get unique boss names from damage_events where is_boss=1
        let boss_names: Vec<String> = de::damage_events
            .filter(de::encounter_id.eq(id))
            .filter(de::is_boss.eq(1))
            .select(de::monster_name)
            .load::<Option<String>>(&mut conn)
            .map_err(|er| er.to_string())?
            .into_iter()
            .filter_map(|n| n)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Get unique player names and class_ids from actor_encounter_stats where is_player=1
        let player_data: Vec<PlayerInfoDto> = s::actor_encounter_stats
            .filter(s::encounter_id.eq(id))
            .filter(s::is_player.eq(1))
            .select((s::name, s::class_id))
            .load::<(Option<String>, Option<i32>)>(&mut conn)
            .map_err(|er| er.to_string())?
            .into_iter()
            .filter_map(|(name, class_id)| name.map(|n| PlayerInfoDto { name: n, class_id }))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        mapped.push(EncounterSummaryDto {
            id,
            started_at_ms: started,
            ended_at_ms: ended,
            total_dmg: td.unwrap_or(0),
            total_heal: th.unwrap_or(0),
            bosses: boss_names,
            players: player_data,
        });
    }

    Ok(RecentEncountersResult {
        rows: mapped,
        total_count,
    })
}

#[tauri::command]
#[specta::specta]
pub fn get_recent_encounters(limit: i32, offset: i32) -> Result<RecentEncountersResult, String> {
    let mut conn = get_conn()?;
    use sch::encounters::dsl as e;
    let rows: Vec<(i32, i64, Option<i64>, Option<i64>, Option<i64>)> = e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .order(e::started_at_ms.desc())
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
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

    for (id, started, ended, td, th) in rows {
        use sch::actor_encounter_stats::dsl as s;
        use sch::damage_events::dsl as de;
        use std::collections::HashSet;

        // Get unique boss names from damage_events where is_boss=1
        let boss_names: Vec<String> = de::damage_events
            .filter(de::encounter_id.eq(id))
            .filter(de::is_boss.eq(1))
            .select(de::monster_name)
            .load::<Option<String>>(&mut conn)
            .map_err(|er| er.to_string())?
            .into_iter()
            .filter_map(|n| n)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Get unique player names and class_ids from actor_encounter_stats where is_player=1
        let player_data: Vec<PlayerInfoDto> = s::actor_encounter_stats
            .filter(s::encounter_id.eq(id))
            .filter(s::is_player.eq(1))
            .select((s::name, s::class_id))
            .load::<(Option<String>, Option<i32>)>(&mut conn)
            .map_err(|er| er.to_string())?
            .into_iter()
            .filter_map(|(name, class_id)| name.map(|n| PlayerInfoDto { name: n, class_id }))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        mapped.push(EncounterSummaryDto {
            id,
            started_at_ms: started,
            ended_at_ms: ended,
            total_dmg: td.unwrap_or(0),
            total_heal: th.unwrap_or(0),
            bosses: boss_names,
            players: player_data,
        });
    }

    Ok(RecentEncountersResult {
        rows: mapped,
        total_count,
    })
}

#[tauri::command]
#[specta::specta]
pub fn get_encounter_actor_stats(encounter_id: i32) -> Result<Vec<ActorEncounterStatDto>, String> {
    let mut conn = get_conn()?;
    use sch::actor_encounter_stats::dsl as s;

    let rows = s::actor_encounter_stats
        .filter(s::encounter_id.eq(encounter_id))
        .filter(s::is_player.eq(1))
        .select((
            s::encounter_id,
            s::actor_id,
            s::name,
            s::class_id,
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
            s::boss_damage_dealt,
            s::boss_hits_dealt,
            s::boss_crit_hits_dealt,
            s::boss_lucky_hits_dealt,
            s::boss_crit_total_dealt,
            s::boss_lucky_total_dealt,
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
        )>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(
            |(
                encounter_id,
                actor_id,
                name,
                class_id,
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
                boss_damage_dealt,
                boss_hits_dealt,
                boss_crit_hits_dealt,
                boss_lucky_hits_dealt,
                boss_crit_total_dealt,
                boss_lucky_total_dealt,
            )| ActorEncounterStatDto {
                encounter_id,
                actor_id,
                name,
                class_id,
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
                boss_damage_dealt,
                boss_hits_dealt,
                boss_crit_hits_dealt,
                boss_lucky_hits_dealt,
                boss_crit_total_dealt,
                boss_lucky_total_dealt,
            },
        )
        .collect())
}

/// Get player name by UID from database
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

#[tauri::command]
#[specta::specta]
pub fn get_recent_players_command(limit: i32) -> Result<Vec<(i64, String)>, String> {
    get_recent_players(limit as i64)
}

#[tauri::command]
#[specta::specta]
pub fn get_player_name_command(uid: i64) -> Result<Option<String>, String> {
    get_name_by_uid(uid)
}

#[tauri::command]
#[specta::specta]
pub fn get_encounter_by_id(encounter_id: i32) -> Result<EncounterSummaryDto, String> {
    let mut conn = get_conn()?;
    use sch::encounters::dsl as e;

    let row: (i32, i64, Option<i64>, Option<i64>, Option<i64>) = e::encounters
        .filter(e::id.eq(encounter_id))
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
        ))
        .first(&mut conn)
        .map_err(|er| er.to_string())?;

    Ok(EncounterSummaryDto {
        id: row.0,
        started_at_ms: row.1,
        ended_at_ms: row.2,
        total_dmg: row.3.unwrap_or(0),
        total_heal: row.4.unwrap_or(0),
        bosses: Vec::new(),
        players: Vec::new(),
    })
}

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

#[tauri::command]
#[specta::specta]
pub fn get_recent_encounters_with_details(
    limit: i32,
    offset: i32,
) -> Result<Vec<EncounterWithDetailsDto>, String> {
    let mut conn = get_conn()?;
    use sch::actor_encounter_stats::dsl as s;
    use sch::damage_events::dsl as de;
    use sch::encounters::dsl as e;
    use std::collections::HashSet;

    // Get encounter summaries
    let encounter_rows: Vec<(i32, i64, Option<i64>, Option<i64>, Option<i64>)> = e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .order(e::started_at_ms.desc())
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
        ))
        .limit(limit as i64)
        .offset(offset as i64)
        .load(&mut conn)
        .map_err(|er| er.to_string())?;

    let mut results = Vec::new();

    for (id, started_at_ms, ended_at_ms, total_dmg, total_heal) in encounter_rows {
        // Get unique boss names from damage_events where is_boss=1
        let boss_names: Vec<String> = de::damage_events
            .filter(de::encounter_id.eq(id))
            .filter(de::is_boss.eq(1))
            .select(de::monster_name)
            .load::<Option<String>>(&mut conn)
            .map_err(|er| er.to_string())?
            .into_iter()
            .filter_map(|n| n)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Get unique player names and class_ids from actor_encounter_stats where is_player=1
        let player_data: Vec<PlayerInfoDto> = s::actor_encounter_stats
            .filter(s::encounter_id.eq(id))
            .filter(s::is_player.eq(1))
            .select((s::name, s::class_id))
            .load::<(Option<String>, Option<i32>)>(&mut conn)
            .map_err(|er| er.to_string())?
            .into_iter()
            .filter_map(|(name, class_id)| name.map(|n| PlayerInfoDto { name: n, class_id }))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        results.push(EncounterWithDetailsDto {
            id,
            started_at_ms,
            ended_at_ms,
            total_dmg: total_dmg.unwrap_or(0),
            total_heal: total_heal.unwrap_or(0),
            bosses: boss_names,
            players: player_data,
        });
    }

    Ok(results)
}

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
    let encounter_row: (i32, i64, Option<i64>, Option<i64>, Option<i64>) = e::encounters
        .filter(e::id.eq(encounter_id))
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
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
