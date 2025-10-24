use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema as sch;
use crate::database::models as m;
use crate::database::{default_db_path, ensure_parent_dir};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSummaryDto {
    pub id: i32,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub total_dmg: i64,
    pub total_heal: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ActorEncounterStatDto {
    pub encounter_id: i32,
    pub actor_id: i64,
    pub name: Option<String>,
    pub is_player: bool,
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
}

fn get_conn() -> Result<diesel::sqlite::SqliteConnection, String> {
    let path = default_db_path();
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy())
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_recent_encounters(limit: i32) -> Result<Vec<EncounterSummaryDto>, String> {
    let mut conn = get_conn()?;
    use sch::encounters::dsl as e;
    let rows: Vec<(i32, i64, Option<i64>, Option<i64>, Option<i64>)> = e::encounters
        .order(e::started_at_ms.desc())
        .select((e::id, e::started_at_ms, e::ended_at_ms, e::total_dmg, e::total_heal))
        .limit(limit as i64)
        .load(&mut conn)
        .map_err(|er| er.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(id, started, ended, td, th)| EncounterSummaryDto {
            id,
            started_at_ms: started,
            ended_at_ms: ended,
            total_dmg: td.unwrap_or(0),
            total_heal: th.unwrap_or(0),
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub fn get_encounter_actor_stats(encounter_id: i32) -> Result<Vec<ActorEncounterStatDto>, String> {
    let mut conn = get_conn()?;
    use sch::actor_encounter_stats::dsl as s;
    use sch::entities::dsl as en;

    let rows = s::actor_encounter_stats
        .inner_join(en::entities.on(en::entity_id.eq(s::actor_id)))
        .filter(s::encounter_id.eq(encounter_id))
        .select((
            s::encounter_id,
            s::actor_id,
            en::name.nullable(),
            en::is_player,
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
        ))
        .order((s::damage_dealt.desc(), s::heal_dealt.desc(), s::damage_taken.desc()))
        .load::<(i32, i64, Option<String>, i32, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64)>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|(
        encounter_id,
        actor_id,
        name_opt,
        is_player_int,
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
    )| ActorEncounterStatDto {
        encounter_id,
        actor_id,
        name: name_opt,
        is_player: is_player_int != 0,
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
    }).collect())
}
