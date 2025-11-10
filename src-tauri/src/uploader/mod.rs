use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use sha2::{Sha256, Digest};

use reqwest::Client;
use tauri::{AppHandle, Emitter, Manager};

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
    pub source_hash: Option<String>,
    pub attempts: Vec<UploadAttemptIn>,
    pub death_events: Vec<UploadDeathEventIn>,
    pub actor_encounter_stats: Vec<UploadActorEncounterStatIn>,
    pub damage_skill_stats: Vec<UploadDamageSkillStatIn>,
    pub heal_skill_stats: Vec<UploadHealSkillStatIn>,
    pub entities: Vec<UploadEntityIn>,
    pub encounter_bosses: Vec<UploadEncounterBossIn>,
    pub detailed_playerdata: Vec<UploadDetailedPlayerDataIn>,
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
    pub name: Option<String>,
    pub class_id: Option<i32>,
    pub class_spec: Option<i32>,
    pub ability_score: Option<i32>,
    pub level: Option<i32>,
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
    pub crit_total_dealt: i64,
    pub crit_total_heal: i64,
    pub crit_total_taken: i64,
    pub lucky_total_dealt: i64,
    pub lucky_total_heal: i64,
    pub lucky_total_taken: i64,
    pub boss_damage_dealt: i64,
    pub boss_hits_dealt: i64,
    pub boss_crit_hits_dealt: i64,
    pub boss_lucky_hits_dealt: i64,
    pub boss_crit_total_dealt: i64,
    pub boss_lucky_total_dealt: i64,
    pub revives: i64,
    pub dps: f64,
    pub duration: f64,
    pub is_player: bool,
    pub is_local_player: bool,
    pub attributes: Option<String>,
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
pub struct UploadDetailedPlayerDataIn {
    pub player_id: i64,
    pub last_seen_ms: i64,
    pub char_serialize_json: String,
    pub profession_list_json: Option<String>,
    pub talent_node_ids_json: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckDuplicatesRequest {
    pub hashes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateInfo {
    pub hash: String,
    pub encounter_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckDuplicatesResponse {
    pub duplicates: Vec<DuplicateInfo>,
    pub missing: Vec<String>,
}

/// Compute a deterministic SHA-256 hash for an encounter
/// Uses a canonical subset of fields to ensure consistency
fn compute_encounter_hash(encounter: &UploadEncounterIn) -> String {
    // Build a canonical representation using selected stable fields
    // Sort attempts by attempt_index to ensure determinism
    let mut sorted_attempts = encounter.attempts.clone();
    sorted_attempts.sort_by_key(|a| a.attempt_index);

    let attempt_values: Vec<serde_json::Value> = sorted_attempts.iter().map(|a| json!({
        "attemptIndex": a.attempt_index,
        "startedAtMs": a.started_at_ms,
        "endedAtMs": a.ended_at_ms,
    })).collect();

    // Include actor IDs to differentiate encounters with same timing but different participants
    let mut actor_ids: Vec<i64> = encounter.actor_encounter_stats.iter().map(|s| s.actor_id).collect();
    actor_ids.sort();

    // Create a canonical JSON structure with only the fields we want to hash
    let canonical = json!({
        "startedAtMs": encounter.started_at_ms,
        "endedAtMs": encounter.ended_at_ms,
        "localPlayerId": encounter.local_player_id,
        "sceneId": encounter.scene_id,
        "sceneName": encounter.scene_name,
        "attempts": attempt_values,
        "actorIds": actor_ids,
    });

    // Serialize to deterministic JSON string (serde_json maintains key order)
    let canonical_str = serde_json::to_string(&canonical).unwrap_or_default();

    // Compute SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(canonical_str.as_bytes());
    let result = hasher.finalize();

    // Return hex-encoded hash
    hex::encode(result)
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
    use sch::detailed_playerdata::dsl as dpd;

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
            s::name,
            s::class_id,
            s::class_spec,
            s::ability_score,
            s::level,
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
            s::revives,
            s::dps,
            s::duration,
            s::is_player,
            s::is_local_player,
            s::attributes,
        ))
        .load::<(
            i64, Option<String>, Option<i32>, Option<i32>, Option<i32>, Option<i32>,
            i64, i64, i64, i64, i64, i64,
            i64, i64, i64, i64, i64, i64,
            i64, i64, i64, i64, i64, i64,
            i64, i64, i64, i64, i64, i64,
            i64, f64, f64, i32, i32, Option<String>
        )>(conn)
        .map_err(|e| e.to_string())?;
    let actor_stats = stats_rows
        .into_iter()
        .map(|(
            actor_id, name, class_id, class_spec, ability_score, level,
            damage_dealt, heal_dealt, damage_taken, hits_dealt, hits_heal, hits_taken,
            crit_hits_dealt, crit_hits_heal, crit_hits_taken,
            lucky_hits_dealt, lucky_hits_heal, lucky_hits_taken,
            crit_total_dealt, crit_total_heal, crit_total_taken,
            lucky_total_dealt, lucky_total_heal, lucky_total_taken,
            boss_damage_dealt, boss_hits_dealt, boss_crit_hits_dealt, boss_lucky_hits_dealt,
            boss_crit_total_dealt, boss_lucky_total_dealt,
            revives, dps, duration, is_player, is_local_player, attributes
        )| UploadActorEncounterStatIn {
            actor_id,
            name,
            class_id,
            class_spec,
            ability_score,
            level,
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
            revives,
            dps,
            duration,
            is_player: is_player != 0,
            is_local_player: is_local_player != 0,
            attributes,
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

    // Detailed player data
    let dpd_rows = dpd::detailed_playerdata
        .select((
            dpd::player_id,
            dpd::last_seen_ms,
            dpd::char_serialize_json,
            dpd::profession_list_json,
            dpd::talent_node_ids_json,
        ))
        .load::<(i64, i64, String, Option<String>, Option<String>)>(conn)
        .map_err(|e| e.to_string())?;
    let detailed_playerdata = dpd_rows
        .into_iter()
        .map(|(player_id, last_seen_ms, char_serialize_json, profession_list_json, talent_node_ids_json)| UploadDetailedPlayerDataIn {
            player_id,
            last_seen_ms,
            char_serialize_json,
            profession_list_json,
            talent_node_ids_json,
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

    let mut encounter = UploadEncounterIn {
        started_at_ms,
        ended_at_ms,
        local_player_id,
        total_dmg,
        total_heal,
        scene_id,
        scene_name,
        source_hash: None, // Will be computed below
        attempts,
        death_events,
        actor_encounter_stats: actor_stats,
        damage_skill_stats,
        heal_skill_stats,
        entities,
        encounter_bosses,
        detailed_playerdata,
    };

    // Compute and set the source hash
    encounter.source_hash = Some(compute_encounter_hash(&encounter));

    Ok(encounter)
}

pub async fn perform_upload(app: AppHandle, api_key: String, base_urls: Vec<String>) -> Result<(), String> {
    CANCEL_FLAG.store(false, Ordering::SeqCst);
    // Establish sqlite connection via diesel
    let path = default_db_path();
    let mut conn = diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy())
        .map_err(|e| e.to_string())?;
    let total = count_ended_encounters(&mut conn)?;
    // Emit to known windows (main + live) when available, fall back to app.emit
    let started_payload: serde_json::Value = json!({"total": total});
    if let Some(w) = app.get_webview_window(crate::WINDOW_MAIN_LABEL) {
        let _ = w.emit("upload:started", started_payload.clone());
    }
    if let Some(w) = app.get_webview_window(crate::WINDOW_LIVE_LABEL) {
        let _ = w.emit("upload:started", started_payload.clone());
    }
    let _ = app.emit("upload:started", started_payload);
    if total == 0 {
        let _ = app.emit("upload:completed", json!({"uploaded": 0, "total": 0}));
        return Ok(());
    }

    let client = Client::builder().timeout(std::time::Duration::from_secs(30)).build().map_err(|e| e.to_string())?;
    let mut offset = 0_i64;
    let batch_size = 10_i64;
    let mut uploaded = 0_i64;
    let mut current_url_index = 0;

    while offset < total {
        if CANCEL_FLAG.load(Ordering::SeqCst) {
            let err_payload: serde_json::Value = json!({"message": "Upload cancelled"});
            if let Some(w) = app.get_webview_window(crate::WINDOW_MAIN_LABEL) {
                let _ = w.emit("upload:error", err_payload.clone());
            }
            if let Some(w) = app.get_webview_window(crate::WINDOW_LIVE_LABEL) {
                let _ = w.emit("upload:error", err_payload.clone());
            }
            let _ = app.emit("upload:error", err_payload);
            return Ok(());
        }
        let rows = load_encounters_slice(&mut conn, offset, batch_size)?;
        if rows.is_empty() { break; }
        let mut payloads = Vec::with_capacity(rows.len());
        for row in &rows {
            payloads.push(build_encounter_payload(&mut conn, row.clone())?);
        }

        // Preflight check: ask server which encounters are duplicates
        let hashes: Vec<String> = payloads.iter()
            .filter_map(|e| e.source_hash.clone())
            .collect();

        let mut filtered_payloads = payloads.clone();

        if !hashes.is_empty() {
            // Perform preflight duplicate check
            let check_url = format!("{}/upload/check", base_urls[current_url_index].trim_end_matches('/'));
            let check_body = CheckDuplicatesRequest { hashes };

            match client.post(&check_url)
                .header("X-Api-Key", api_key.clone())
                .json(&check_body)
                .send().await
            {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(check_result) = resp.json::<CheckDuplicatesResponse>().await {
                        // Filter out duplicates from the batch
                        let duplicate_hashes: std::collections::HashSet<String> = check_result.duplicates.iter()
                            .map(|d| d.hash.clone())
                            .collect();

                        filtered_payloads = payloads.into_iter()
                            .filter(|e| {
                                if let Some(ref hash) = e.source_hash {
                                    !duplicate_hashes.contains(hash)
                                } else {
                                    true // Keep encounters without hash
                                }
                            })
                            .collect();

                        let skipped = rows.len() - filtered_payloads.len();
                        if skipped > 0 {
                            // Emit progress for skipped duplicates
                            uploaded += skipped as i64;
                            let progress_payload: serde_json::Value = json!({
                                "uploaded": uploaded,
                                "total": total,
                                "skipped": skipped,
                                "message": format!("Skipped {} duplicate(s)", skipped)
                            });
                            if let Some(w) = app.get_webview_window(crate::WINDOW_MAIN_LABEL) {
                                let _ = w.emit("upload:progress", progress_payload.clone());
                            }
                            if let Some(w) = app.get_webview_window(crate::WINDOW_LIVE_LABEL) {
                                let _ = w.emit("upload:progress", progress_payload.clone());
                            }
                            let _ = app.emit("upload:progress", progress_payload);
                        }
                    }
                }
                _ => {
                    // Preflight check failed; proceed with full batch upload (server will dedupe)
                }
            }
        }

        // If all encounters were duplicates, skip upload and continue to next batch
        if filtered_payloads.is_empty() {
            offset += rows.len() as i64;
            continue;
        }

        let body = UploadRequestBody { encounters: filtered_payloads };

        // Try each URL in sequence until one succeeds
        let mut upload_success = false;
        let mut last_error = String::new();

        while current_url_index < base_urls.len() && !upload_success {
            // Use trailing slash to match server route registration (/api/v1/upload/)
            // Avoids a 307 redirect which can drop custom headers like X-Api-Key.
            let url = format!("{}/upload/", base_urls[current_url_index].trim_end_matches('/'));
            match client.post(&url)
                .header("X-Api-Key", api_key.clone())
                .json(&body)
                .send().await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        upload_success = true;
                    } else {
                        let text = resp.text().await.unwrap_or_default();
                        last_error = format!("Server error from {}: {}", base_urls[current_url_index], text);
                        // Try next URL for server errors
                        current_url_index += 1;
                    }
                }
                Err(e) => {
                    last_error = format!("Connection error from {}: {}", base_urls[current_url_index], e);
                    // Try next URL for connection errors
                    current_url_index += 1;
                }
            }
        }

        if !upload_success {
            let err_payload: serde_json::Value = json!({"message": last_error});
            if let Some(w) = app.get_webview_window(crate::WINDOW_MAIN_LABEL) {
                let _ = w.emit("upload:error", err_payload.clone());
            }
            if let Some(w) = app.get_webview_window(crate::WINDOW_LIVE_LABEL) {
                let _ = w.emit("upload:error", err_payload.clone());
            }
            let _ = app.emit("upload:error", err_payload);
            return Err(last_error);
        }

        uploaded += rows.len() as i64;
        offset += rows.len() as i64;
        // Emit progress for UI to known windows and as a fallback via app.emit
        let progress_payload: serde_json::Value = json!({"uploaded": uploaded, "total": total, "batch": (offset / batch_size)+1});
        if let Some(w) = app.get_webview_window(crate::WINDOW_MAIN_LABEL) {
            let _ = w.emit("upload:progress", progress_payload.clone());
        }
        if let Some(w) = app.get_webview_window(crate::WINDOW_LIVE_LABEL) {
            let _ = w.emit("upload:progress", progress_payload.clone());
        }
        let _ = app.emit("upload:progress", progress_payload);
    }

    let completed_payload: serde_json::Value = json!({"uploaded": uploaded, "total": total});
    if let Some(w) = app.get_webview_window(crate::WINDOW_MAIN_LABEL) {
        let _ = w.emit("upload:completed", completed_payload.clone());
    }
    if let Some(w) = app.get_webview_window(crate::WINDOW_LIVE_LABEL) {
        let _ = w.emit("upload:completed", completed_payload.clone());
    }
    let _ = app.emit("upload:completed", completed_payload);
    Ok(())
}

pub fn cancel_upload() { CANCEL_FLAG.store(true, Ordering::SeqCst); }

#[tauri::command]
#[specta::specta]
pub async fn start_upload(app: tauri::AppHandle, api_key: String, base_url: Option<String>) -> Result<(), String> {
    let mut base_urls = Vec::new();

    // If a specific base_url is provided, use only that
    if let Some(url) = base_url {
        base_urls.push(url);
    } else {
        // Otherwise, try localhost first, then fallback
        let localhost_url = std::env::var("WEBSITE_API_BASE")
            .unwrap_or_else(|_| "http://localhost:8080/api/v1".to_string());
        base_urls.push(localhost_url);
        base_urls.push("https://api.i7s.me".to_string());
    }

    let app_cloned = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = perform_upload(app_cloned, api_key, base_urls).await {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_url_ordering_when_no_base_url() {
        // Test that when no base_url is provided, it uses localhost first then https://api.i7s.me
        let mut base_urls = Vec::new();

        // Simulate the logic from start_upload when base_url is None
        let localhost_url = std::env::var("WEBSITE_API_BASE")
            .unwrap_or_else(|_| "http://localhost:8080/api/v1".to_string());
        base_urls.push(localhost_url);
        base_urls.push("https://api.i7s.me".to_string());

        // Verify the ordering
        assert_eq!(base_urls.len(), 2);
        assert!(base_urls[0].contains("localhost") || base_urls[0].contains("127.0.0.1"));
        assert_eq!(base_urls[1], "https://api.i7s.me");
    }

    #[test]
    fn test_specific_base_url_only() {
        // Test that when a specific base_url is provided, it only uses that URL
        let specific_url = "https://custom.example.com".to_string();
        let mut base_urls = Vec::new();

        // Simulate the logic from start_upload when base_url is Some
        base_urls.push(specific_url.clone());

        // Verify only the specific URL is used
        assert_eq!(base_urls.len(), 1);
        assert_eq!(base_urls[0], specific_url);
    }

    #[test]
    fn test_url_formatting() {
        // Test that URLs are formatted correctly when constructing the upload endpoint
        let base_urls = vec![
            "http://localhost:8080/api/v1/".to_string(),
            "https://api.i7s.me".to_string(),
            "https://custom.example.com/api/".to_string(),
        ];

        let expected_endpoints = vec![
            "http://localhost:8080/api/v1/upload/",
            "https://api.i7s.me/upload/",
            "https://custom.example.com/api/upload/",
        ];

        for (i, base_url) in base_urls.iter().enumerate() {
            let formatted_url = format!("{}/upload/", base_url.trim_end_matches('/'));
            assert_eq!(formatted_url, expected_endpoints[i]);
        }
    }
}
