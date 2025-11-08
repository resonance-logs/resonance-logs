use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};

use reqwest::Client;
use tauri::{AppHandle, Emitter};

use crate::database::{default_db_path};
use diesel::prelude::*;
use crate::database::schema as sch;

static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadEncounterIn {
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub local_player_id: Option<i64>,
    pub total_dmg: Option<i64>,
    pub total_heal: Option<i64>,
    pub scene_id: Option<i32>,
    pub scene_name: Option<String>,
    pub attempts: Vec<UploadAttemptIn>,
    pub death_events: Vec<UploadDeathEventIn>,
    pub actor_encounter_stats: Vec<UploadActorEncounterStatIn>,
    pub damage_skill_stats: Vec<UploadDamageSkillStatIn>,
    pub heal_skill_stats: Vec<UploadHealSkillStatIn>,
    pub entities: Vec<UploadEntityIn>,
    pub encounter_bosses: Vec<UploadEncounterBossIn>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadAttemptIn {
    pub attempt_index: i32,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub reason: String,
    pub boss_hp_start: Option<i64>,
    pub boss_hp_end: Option<i64>,
    pub total_deaths: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadDeathEventIn {
    pub timestamp_ms: i64,
    pub actor_id: i64,
    pub killer_id: Option<i64>,
    pub skill_id: Option<i32>,
    pub is_local_player: bool,
    pub attempt_index: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadActorEncounterStatIn {
    pub actor_id: i64,
    pub class_spec: Option<i32>,
    pub damage_dealt: i64,
    pub heal_dealt: i64,
    pub damage_taken: i64,
    pub hits_dealt: i64,
    pub hits_heal: i64,
    pub hits_taken: i64,
    pub name: Option<String>,
    pub class_id: Option<i32>,
    pub ability_score: Option<i32>,
    pub level: Option<i32>,
    pub is_player: bool,
    pub is_local_player: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadDamageSkillStatIn {
    pub attacker_id: i64,
    pub defender_id: Option<i64>,
    pub skill_id: i32,
    pub hits: i32,
    pub total_value: i64,
    pub crit_hits: i32,
    pub lucky_hits: i32,
    pub crit_total: i64,
    pub lucky_total: i64,
    pub hp_loss_total: i64,
    pub shield_loss_total: i64,
    pub monster_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadHealSkillStatIn {
    pub healer_id: i64,
    pub target_id: Option<i64>,
    pub skill_id: i32,
    pub hits: i32,
    pub total_value: i64,
    pub crit_hits: i32,
    pub lucky_hits: i32,
    pub crit_total: i64,
    pub lucky_total: i64,
    pub monster_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadEntityIn {
    pub entity_id: i64,
    pub name: Option<String>,
    pub class_id: Option<i32>,
    pub class_spec: Option<i32>,
    pub ability_score: Option<i32>,
    pub level: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadEncounterBossIn {
    pub monster_name: String,
    pub hits: i32,
    pub total_damage: i64,
    pub max_hp: Option<i64>,
    pub is_defeated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadRequestBody {
    pub encounters: Vec<UploadEncounterIn>,
}

/// Counts ended encounters eligible for upload.
fn count_ended_encounters(conn: &mut diesel::sqlite::SqliteConnection) -> Result<i64, String> {
    use sch::encounters::dsl as e;
    e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .count()
        .get_result(conn)
        .map_err(|er| er.to_string())
}

/// Load a slice of ended encounters for upload.
fn load_encounters_slice(conn: &mut diesel::sqlite::SqliteConnection, offset: i64, limit: i64) -> Result<Vec<(i32, i64, Option<i64>, Option<i64>, Option<i64>, Option<i32>, Option<String>, Option<i64>)>, String> {
    use sch::encounters::dsl as e;
    e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .order(e::started_at_ms.asc())
        .select((
            e::id,
            e::started_at_ms,
            e::ended_at_ms,
            e::total_dmg,
            e::total_heal,
            e::scene_id,
            e::scene_name,
            e::local_player_id,
        ))
        .offset(offset)
        .limit(limit)
        .load::<(i32, i64, Option<i64>, Option<i64>, Option<i64>, Option<i32>, Option<String>, Option<i64>)>(conn)
        .map_err(|er| er.to_string())
}

fn build_encounter_payload(conn: &mut diesel::sqlite::SqliteConnection, row: (i32, i64, Option<i64>, Option<i64>, Option<i64>, Option<i32>, Option<String>, Option<i64>)) -> Result<UploadEncounterIn, String> {
    let (id, started_at_ms, ended_at_ms, total_dmg, total_heal, scene_id, scene_name, local_player_id) = row;
    use sch::attempts::dsl as a;
    use sch::death_events::dsl as de;
    use sch::actor_encounter_stats::dsl as s;
    use sch::damage_skill_stats::dsl as dss;
    use sch::heal_skill_stats::dsl as hss;
    use sch::encounter_bosses::dsl as eb;
    use sch::entities::dsl as en;

    // Attempts
    let attempts_rows = a::attempts
        .filter(a::encounter_id.eq(id))
        .order(a::attempt_index.asc())
        .select((
            a::attempt_index,
            a::started_at_ms,
            a::ended_at_ms,
            a::reason,
            a::boss_hp_start,
            a::boss_hp_end,
            a::total_deaths,
        ))
        // Tuple must match select order and arity (7 columns)
        .load::<(i32, i64, Option<i64>, String, Option<i64>, Option<i64>, i32)>(conn)
        .map_err(|e| e.to_string())?;
    let attempts = attempts_rows
        .into_iter()
        .map(|(attempt_index, started_at_ms, ended_at_ms, reason, boss_hp_start, boss_hp_end, total_deaths)| UploadAttemptIn {
            attempt_index,
            started_at_ms,
            ended_at_ms,
            reason,
            boss_hp_start,
            boss_hp_end,
            total_deaths,
        })
        .collect::<Vec<_>>();

    // Death events
    let de_rows = de::death_events
        .filter(de::encounter_id.eq(id))
        .select((
            de::timestamp_ms,
            de::actor_id,
            de::killer_id,
            de::skill_id,
            de::is_local_player,
            de::attempt_index,
        ))
        .load::<(i64, i64, Option<i64>, Option<i32>, i32, Option<i32>)>(conn)
        .map_err(|e| e.to_string())?;
    let death_events = de_rows
        .into_iter()
        .map(|(timestamp_ms, actor_id, killer_id, skill_id, is_local_player, attempt_index)| UploadDeathEventIn {
            timestamp_ms,
            actor_id,
            killer_id,
            skill_id,
            is_local_player: is_local_player != 0,
            attempt_index,
        })
        .collect::<Vec<_>>();

    // Actor encounter stats
    let stats_rows = s::actor_encounter_stats
        .filter(s::encounter_id.eq(id))
        .select((
            s::actor_id,
            s::class_spec,
            s::damage_dealt,
            s::heal_dealt,
            s::damage_taken,
            s::hits_dealt,
            s::hits_heal,
            s::hits_taken,
            s::name,
            s::class_id,
            s::ability_score,
            s::level,
            s::is_player,
            s::is_local_player,
        ))
        .load::<(i64, Option<i32>, i64, i64, i64, i64, i64, i64, Option<String>, Option<i32>, Option<i32>, Option<i32>, i32, i32)>(conn)
        .map_err(|e| e.to_string())?;
    let actor_stats = stats_rows
        .into_iter()
        .map(|(actor_id, class_spec, damage_dealt, heal_dealt, damage_taken, hits_dealt, hits_heal, hits_taken, name, class_id, ability_score, level, is_player, is_local_player)| UploadActorEncounterStatIn {
            actor_id,
            class_spec,
            damage_dealt,
            heal_dealt,
            damage_taken,
            hits_dealt,
            hits_heal,
            hits_taken,
            name,
            class_id,
            ability_score,
            level,
            is_player: is_player != 0,
            is_local_player: is_local_player != 0,
        })
        .collect::<Vec<_>>();

    // Damage skill stats
    let dmg_rows = dss::damage_skill_stats
        .filter(dss::encounter_id.eq(id))
        .select((
            dss::attacker_id,
            dss::defender_id,
            dss::skill_id,
            dss::hits,
            dss::total_value,
            dss::crit_hits,
            dss::lucky_hits,
            dss::crit_total,
            dss::lucky_total,
            dss::hp_loss_total,
            dss::shield_loss_total,
            dss::monster_name,
        ))
        .load::<(i64, Option<i64>, i32, i32, i64, i32, i32, i64, i64, i64, i64, Option<String>)>(conn)
        .map_err(|e| e.to_string())?;
    let damage_skill_stats = dmg_rows
        .into_iter()
        .map(|(attacker_id, defender_id, skill_id, hits, total_value, crit_hits, lucky_hits, crit_total, lucky_total, hp_loss_total, shield_loss_total, monster_name)| UploadDamageSkillStatIn {
            attacker_id,
            defender_id,
            skill_id,
            hits,
            total_value,
            crit_hits,
            lucky_hits,
            crit_total,
            lucky_total,
            hp_loss_total,
            shield_loss_total,
            monster_name,
        })
        .collect::<Vec<_>>();

    // Heal skill stats
    let heal_rows = hss::heal_skill_stats
        .filter(hss::encounter_id.eq(id))
        .select((
            hss::healer_id,
            hss::target_id,
            hss::skill_id,
            hss::hits,
            hss::total_value,
            hss::crit_hits,
            hss::lucky_hits,
            hss::crit_total,
            hss::lucky_total,
            hss::monster_name,
        ))
        .load::<(i64, Option<i64>, i32, i32, i64, i32, i32, i64, i64, Option<String>)>(conn)
        .map_err(|e| e.to_string())?;
    let heal_skill_stats = heal_rows
        .into_iter()
        .map(|(healer_id, target_id, skill_id, hits, total_value, crit_hits, lucky_hits, crit_total, lucky_total, monster_name)| UploadHealSkillStatIn {
            healer_id,
            target_id,
            skill_id,
            hits,
            total_value,
            crit_hits,
            lucky_hits,
            crit_total,
            lucky_total,
            monster_name,
        })
        .collect::<Vec<_>>();

    // Encounter bosses
    let boss_rows = eb::encounter_bosses
        .filter(eb::encounter_id.eq(id))
        .select((eb::monster_name, eb::hits, eb::total_damage, eb::max_hp, eb::is_defeated))
        .load::<(String, i32, i64, Option<i64>, i32)>(conn)
        .map_err(|e| e.to_string())?;
    let encounter_bosses = boss_rows
        .into_iter()
        .map(|(monster_name, hits, total_damage, max_hp, is_defeated)| UploadEncounterBossIn {
            monster_name,
            hits,
            total_damage,
            max_hp,
            is_defeated: is_defeated != 0,
        })
        .collect::<Vec<_>>();

    // Entities snapshot (players only where is_player=1 or referenced actors)
    let entity_rows = en::entities
        .select((
            en::entity_id,
            en::name,
            en::class_id,
            en::class_spec,
            en::ability_score,
            en::level,
        ))
        .load::<(i64, Option<String>, Option<i32>, Option<i32>, Option<i32>, Option<i32>)>(conn)
        .map_err(|e| e.to_string())?;
    let mut entities = Vec::new();
    for (entity_id, name, class_id, class_spec, ability_score, level) in entity_rows {
        entities.push(UploadEntityIn { entity_id, name, class_id, class_spec, ability_score, level });
    }

    Ok(UploadEncounterIn {
        started_at_ms,
        ended_at_ms,
        local_player_id,
        total_dmg,
        total_heal,
        scene_id,
        scene_name,
        attempts,
        death_events,
        actor_encounter_stats: actor_stats,
        damage_skill_stats,
        heal_skill_stats,
        entities,
        encounter_bosses,
    })
}

pub async fn perform_upload(app: AppHandle, api_key: String, base_url: String) -> Result<(), String> {
    CANCEL_FLAG.store(false, Ordering::SeqCst);
    // Establish sqlite connection via diesel
    let path = default_db_path();
    let mut conn = diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy())
        .map_err(|e| e.to_string())?;
    let total = count_ended_encounters(&mut conn)?;
    let _ = app.emit("upload:started", json!({"total": total}));
    if total == 0 {
        let _ = app.emit("upload:completed", json!({"uploaded": 0, "total": 0}));
        return Ok(());
    }

    let client = Client::builder().timeout(std::time::Duration::from_secs(30)).build().map_err(|e| e.to_string())?;
    let mut offset = 0_i64;
    let batch_size = 10_i64;
    let mut uploaded = 0_i64;

    while offset < total {
        if CANCEL_FLAG.load(Ordering::SeqCst) {
            let _ = app.emit("upload:error", json!({"message": "Upload cancelled"}));
            return Ok(());
        }
        let rows = load_encounters_slice(&mut conn, offset, batch_size)?;
        if rows.is_empty() { break; }
        let mut payloads = Vec::with_capacity(rows.len());
        for row in &rows {
            payloads.push(build_encounter_payload(&mut conn, row.clone())?);
        }
        let body = UploadRequestBody { encounters: payloads };
        let url = format!("{}/upload/encounters", base_url.trim_end_matches('/'));
        let resp = client.post(&url)
            .header("X-Api-Key", api_key.clone())
            .json(&body)
            .send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            let _ = app.emit("upload:error", json!({"message": format!("Server error: {}", text)}));
            return Err(format!("Server returned {}", text));
        }
        uploaded += rows.len() as i64;
        offset += rows.len() as i64;
        let _ = app.emit("upload:progress", json!({"uploaded": uploaded, "total": total, "batch": (offset / batch_size)+1}));
    }

    let _ = app.emit("upload:completed", json!({"uploaded": uploaded, "total": total}));
    Ok(())
}

pub fn cancel_upload() { CANCEL_FLAG.store(true, Ordering::SeqCst); }

#[tauri::command]
#[specta::specta]
pub async fn start_upload(app: tauri::AppHandle, api_key: String, base_url: Option<String>) -> Result<(), String> {
    let base = base_url
        .or_else(|| std::env::var("WEBSITE_API_BASE").ok())
        .unwrap_or_else(|| "http://localhost:8080/api/v1".to_string());
    let app_cloned = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = perform_upload(app_cloned, api_key, base).await {
            // best-effort error event
            let _ = app.emit("upload:error", json!({"message": e}));
        }
    });
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn cancel_upload_cmd() -> Result<(), String> {
    cancel_upload();
    Ok(())
}
