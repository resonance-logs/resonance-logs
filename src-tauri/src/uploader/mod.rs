pub mod player_data_sync;

use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use reqwest::Client;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Notify, RwLock};
use tokio::time::{Duration, interval};

use crate::database::schema as sch;
use crate::database::{default_db_path, now_ms};
use diesel::prelude::*;

static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);
static UPLOAD_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
const AUTO_UPLOAD_INTERVAL_SECS: u64 = 30;

/// Settings from the moduleSync.json store file
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModuleSyncSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_auto_upload")]
    pub auto_upload: bool,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
}

fn default_auto_upload() -> bool {
    true
}

/// Read the moduleSync settings from the tauri-plugin-svelte store file
/// This is a blocking operation - call from spawn_blocking or non-async context
pub(crate) fn read_module_sync_settings_blocking(
    settings_path: std::path::PathBuf,
) -> Option<ModuleSyncSettings> {
    let content = std::fs::read_to_string(&settings_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Get the path to the moduleSync.json settings file
pub(crate) fn get_module_sync_settings_path(app: &AppHandle) -> Option<std::path::PathBuf> {
    let app_data_dir = app.path().app_config_dir().ok()?;
    Some(
        app_data_dir
            .join("tauri-plugin-svelte")
            .join("moduleSync.json"),
    )
}

#[derive(Clone, Default)]
pub struct AutoUploadState {
    api_key: Arc<RwLock<Option<String>>>,
    base_url: Arc<RwLock<Option<String>>>,
    auto_upload_enabled: Arc<RwLock<bool>>,
    notifier: Arc<Notify>,
}

impl AutoUploadState {
    pub async fn sync_from_settings(
        &self,
        api_key: Option<String>,
        base_url: Option<String>,
        auto_upload: bool,
    ) {
        let sanitized_key = api_key.and_then(|k| {
            let trimmed = k.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });
        let sanitized_base = base_url.and_then(|url| {
            let trimmed = url.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

        *self.api_key.write().await = sanitized_key;
        *self.base_url.write().await = sanitized_base;
        *self.auto_upload_enabled.write().await = auto_upload;
        self.notifier.notify_one();
    }

    /// Reload settings from the JSON file (non-blocking)
    pub async fn reload_from_file(&self, app: &AppHandle) {
        let settings_path = match get_module_sync_settings_path(app) {
            Some(p) => p,
            None => {
                log::debug!("could not determine moduleSync.json settings path");
                return;
            }
        };

        // Read file in a blocking thread to avoid blocking the async runtime
        let settings = match tauri::async_runtime::spawn_blocking(move || {
            read_module_sync_settings_blocking(settings_path)
        })
        .await
        {
            Ok(s) => s,
            Err(e) => {
                log::debug!("failed to spawn blocking task for settings read: {}", e);
                return;
            }
        };

        if let Some(settings) = settings {
            let api_key = if settings.api_key.trim().is_empty() {
                None
            } else {
                Some(settings.api_key.trim().to_string())
            };
            let base_url = if settings.base_url.trim().is_empty() {
                None
            } else {
                Some(settings.base_url.trim().to_string())
            };
            let has_key = api_key.is_some();
            *self.api_key.write().await = api_key;
            *self.base_url.write().await = base_url;
            *self.auto_upload_enabled.write().await = settings.auto_upload;
            log::debug!(
                "reloaded auto upload settings: auto_upload={}, has_api_key={}",
                settings.auto_upload,
                has_key
            );
        } else {
            log::debug!("could not read moduleSync.json settings file");
        }
    }

    pub async fn current_api_key(&self) -> Option<String> {
        self.api_key.read().await.clone()
    }

    pub async fn current_base_url(&self) -> Option<String> {
        self.base_url.read().await.clone()
    }

    pub async fn is_auto_upload_enabled(&self) -> bool {
        *self.auto_upload_enabled.read().await
    }

    pub fn notifier(&self) -> Arc<Notify> {
        self.notifier.clone()
    }
}

struct UploadRunGuard;

impl UploadRunGuard {
    fn try_acquire() -> Option<Self> {
        if UPLOAD_IN_PROGRESS
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            Some(Self)
        } else {
            None
        }
    }
}

impl Drop for UploadRunGuard {
    fn drop(&mut self) {
        UPLOAD_IN_PROGRESS.store(false, Ordering::SeqCst);
    }
}

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
    pub dungeon_segments: Vec<UploadDungeonSegmentIn>,
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
    pub hit_details: Option<String>,
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
    pub heal_details: Option<String>,
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
pub struct UploadDungeonSegmentIn {
    pub segment_type: String,
    pub boss_entity_id: Option<i64>,
    pub boss_monster_type_id: Option<i64>,
    pub boss_name: Option<String>,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    pub total_damage: i64,
    pub hit_count: i64,
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
#[serde(rename_all = "camelCase")]
pub struct UploadRequestBody {
    pub encounters: Vec<UploadEncounterIn>,
    pub client_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadEncountersResponse {
    pub ingested: i32,
    pub ids: Vec<i64>,
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

fn resolve_base_urls(base_url: Option<String>) -> Vec<String> {
    // Ignore any user-provided base_url from settings. Always try the
    // public API first and fall back to the local development server.
    // This removes the user-configurable module sync URL from advanced
    // settings and enforces a fixed primary+fallback ordering.
    vec![
        "https://api.bpsr.app/api/v1".to_string(),
        "http://localhost:8080/api/v1".to_string(),
    ]
}

fn mark_encounters_uploaded(
    conn: &mut diesel::sqlite::SqliteConnection,
    ids: &[i32],
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    use sch::encounters::dsl as e;
    diesel::update(e::encounters.filter(e::id.eq_any(ids)))
        .set(e::uploaded_at_ms.eq(now_ms()))
        .execute(conn)
        .map(|_| ())
        .map_err(|er| er.to_string())
}

fn pending_encounter_count() -> Result<i64, String> {
    let path = default_db_path();
    let mut conn = diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy())
        .map_err(|e| e.to_string())?;
    count_ended_encounters(&mut conn)
}

pub fn start_auto_upload_task(app: AppHandle, state: AutoUploadState) {
    tauri::async_runtime::spawn(async move {
        // Load settings once at startup
        state.reload_from_file(&app).await;

        let mut ticker = interval(Duration::from_secs(AUTO_UPLOAD_INTERVAL_SECS));
        let notifier = state.notifier();
        loop {
            tokio::select! {
                _ = ticker.tick() => {},
                _ = notifier.notified() => {
                    // Settings changed notification - reload settings
                    state.reload_from_file(&app).await;
                },
            }

            if let Err(err) = maybe_trigger_auto_upload(app.clone(), state.clone()).await {
                log::warn!("auto upload check failed: {}", err);
            }
        }
    });
}

async fn maybe_trigger_auto_upload(app: AppHandle, state: AutoUploadState) -> Result<(), String> {
    // Check if auto-upload is enabled (uses cached value)
    if !state.is_auto_upload_enabled().await {
        return Ok(());
    }

    let api_key = match state.current_api_key().await {
        Some(key) => key,
        None => return Ok(()),
    };

    let pending = tauri::async_runtime::spawn_blocking(|| pending_encounter_count())
        .await
        .map_err(|e| e.to_string())??;

    if pending == 0 {
        return Ok(());
    }

    log::info!(
        "auto upload: found {} pending encounter(s), starting upload",
        pending
    );

    if let Some(guard) = UploadRunGuard::try_acquire() {
        let base_urls = resolve_base_urls(state.current_base_url().await);
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            let _guard = guard;
            if let Err(e) = perform_upload(app_clone.clone(), api_key, base_urls).await {
                let _ = app_clone.emit("upload:error", json!({"message": e}));
            }
        });
    }

    Ok(())
}

/// Compute a deterministic SHA-256 hash for an encounter
/// Uses a canonical subset of fields to ensure consistency
fn compute_encounter_hash(encounter: &UploadEncounterIn) -> String {
    // Build a canonical representation using selected stable fields
    // Sort attempts by attempt_index to ensure determinism
    let mut sorted_attempts = encounter.attempts.clone();
    sorted_attempts.sort_by_key(|a| a.attempt_index);

    let attempt_values: Vec<serde_json::Value> = sorted_attempts
        .iter()
        .map(|a| {
            json!({
                "attemptIndex": a.attempt_index,
                "startedAtMs": a.started_at_ms,
                "endedAtMs": a.ended_at_ms,
            })
        })
        .collect();

    // Include actor IDs to differentiate encounters with same timing but different participants
    let mut actor_ids: Vec<i64> = encounter
        .actor_encounter_stats
        .iter()
        .map(|s| s.actor_id)
        .collect();
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
        .filter(e::uploaded_at_ms.is_null())
        .count()
        .get_result(conn)
        .map_err(|er| er.to_string())
}

/// Load a slice of ended encounters for upload.
fn load_encounters_slice(
    conn: &mut diesel::sqlite::SqliteConnection,
    offset: i64,
    limit: i64,
) -> Result<
    Vec<(
        i32,
        i64,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i32>,
        Option<String>,
        Option<i64>,
    )>,
    String,
> {
    use sch::encounters::dsl as e;
    e::encounters
        .filter(e::ended_at_ms.is_not_null())
        .filter(e::uploaded_at_ms.is_null())
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
        .load::<(
            i32,
            i64,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<i32>,
            Option<String>,
            Option<i64>,
        )>(conn)
        .map_err(|er| er.to_string())
}

fn build_encounter_payload(
    conn: &mut diesel::sqlite::SqliteConnection,
    row: (
        i32,
        i64,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i32>,
        Option<String>,
        Option<i64>,
    ),
) -> Result<UploadEncounterIn, String> {
    let (
        id,
        started_at_ms,
        ended_at_ms,
        total_dmg,
        total_heal,
        scene_id,
        scene_name,
        local_player_id,
    ) = row;
    use sch::actor_encounter_stats::dsl as s;
    use sch::attempts::dsl as a;
    use sch::damage_skill_stats::dsl as dss;
    use sch::death_events::dsl as de;
    use sch::detailed_playerdata::dsl as dpd;
    use sch::dungeon_segments::dsl as ds;
    use sch::encounter_bosses::dsl as eb;
    use sch::entities::dsl as en;
    use sch::heal_skill_stats::dsl as hss;

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
        .map(
            |(
                attempt_index,
                started_at_ms,
                ended_at_ms,
                reason,
                boss_hp_start,
                boss_hp_end,
                total_deaths,
            )| UploadAttemptIn {
                attempt_index,
                started_at_ms,
                ended_at_ms,
                reason,
                boss_hp_start,
                boss_hp_end,
                total_deaths,
            },
        )
        .collect::<Vec<_>>();

    // Dungeon segments
    let segment_rows = ds::dungeon_segments
        .filter(ds::encounter_id.eq(id))
        .order(ds::started_at_ms.asc())
        .select((
            ds::segment_type,
            ds::boss_entity_id,
            ds::boss_monster_type_id,
            ds::boss_name,
            ds::started_at_ms,
            ds::ended_at_ms,
            ds::total_damage,
            ds::hit_count,
        ))
        .load::<(
            String,
            Option<i64>,
            Option<i64>,
            Option<String>,
            i64,
            Option<i64>,
            i64,
            i64,
        )>(conn)
        .map_err(|e| e.to_string())?;
    let dungeon_segments = segment_rows
        .into_iter()
        .map(
            |(
                segment_type,
                boss_entity_id,
                boss_monster_type_id,
                boss_name,
                started_at_ms,
                ended_at_ms,
                total_damage,
                hit_count,
            )| UploadDungeonSegmentIn {
                segment_type,
                boss_entity_id,
                boss_monster_type_id,
                boss_name,
                started_at_ms,
                ended_at_ms,
                total_damage,
                hit_count,
            },
        )
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
        .map(
            |(timestamp_ms, actor_id, killer_id, skill_id, is_local_player, attempt_index)| {
                UploadDeathEventIn {
                    timestamp_ms,
                    actor_id,
                    killer_id,
                    skill_id,
                    is_local_player: is_local_player != 0,
                    attempt_index,
                }
            },
        )
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
            i64,
            Option<String>,
            Option<i32>,
            Option<i32>,
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
            i64,
            f64,
            f64,
            i32,
            i32,
            Option<String>,
        )>(conn)
        .map_err(|e| e.to_string())?;
    let actor_stats = stats_rows
        .into_iter()
        .map(
            |(
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
                is_player,
                is_local_player,
                attributes,
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
            },
        )
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
            dss::hit_details,
            dss::monster_name,
        ))
        .load::<(
            i64,
            Option<i64>,
            i32,
            i32,
            i64,
            i32,
            i32,
            i64,
            i64,
            i64,
            i64,
            String,
            Option<String>,
        )>(conn)
        .map_err(|e| e.to_string())?;
    let damage_skill_stats = dmg_rows
        .into_iter()
        .map(
            |(
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
                hit_details,
                monster_name,
            )| UploadDamageSkillStatIn {
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
                hit_details: Some(hit_details),
                monster_name,
            },
        )
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
            hss::heal_details,
            hss::monster_name,
        ))
        .load::<(
            i64,
            Option<i64>,
            i32,
            i32,
            i64,
            i32,
            i32,
            i64,
            i64,
            String,
            Option<String>,
        )>(conn)
        .map_err(|e| e.to_string())?;
    let heal_skill_stats = heal_rows
        .into_iter()
        .map(
            |(
                healer_id,
                target_id,
                skill_id,
                hits,
                total_value,
                crit_hits,
                lucky_hits,
                crit_total,
                lucky_total,
                heal_details,
                monster_name,
            )| UploadHealSkillStatIn {
                healer_id,
                target_id,
                skill_id,
                hits,
                total_value,
                crit_hits,
                lucky_hits,
                crit_total,
                lucky_total,
                heal_details: Some(heal_details),
                monster_name,
            },
        )
        .collect::<Vec<_>>();

    // Encounter bosses
    let boss_rows = eb::encounter_bosses
        .filter(eb::encounter_id.eq(id))
        .select((
            eb::monster_name,
            eb::hits,
            eb::total_damage,
            eb::max_hp,
            eb::is_defeated,
        ))
        .load::<(String, i32, i64, Option<i64>, i32)>(conn)
        .map_err(|e| e.to_string())?;
    let encounter_bosses = boss_rows
        .into_iter()
        .map(
            |(monster_name, hits, total_damage, max_hp, is_defeated)| UploadEncounterBossIn {
                monster_name,
                hits,
                total_damage,
                max_hp,
                is_defeated: is_defeated != 0,
            },
        )
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
        .map(
            |(
                player_id,
                last_seen_ms,
                char_serialize_json,
                profession_list_json,
                talent_node_ids_json,
            )| UploadDetailedPlayerDataIn {
                player_id,
                last_seen_ms,
                char_serialize_json,
                profession_list_json,
                talent_node_ids_json,
            },
        )
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
        .load::<(
            i64,
            Option<String>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
        )>(conn)
        .map_err(|e| e.to_string())?;
    let mut entities = Vec::new();
    for (entity_id, name, class_id, class_spec, ability_score, level) in entity_rows {
        entities.push(UploadEntityIn {
            entity_id,
            name,
            class_id,
            class_spec,
            ability_score,
            level,
        });
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
        dungeon_segments,
    };

    // Compute and set the source hash
    encounter.source_hash = Some(compute_encounter_hash(&encounter));

    Ok(encounter)
}

// Scenes allowed to be uploaded to the public website and their minimum
// required boss max HP. If the scene id isn't present here the encounter
// will be skipped by the uploader.
fn allowed_scenes_min_hp() -> Vec<(i32, i64)> {
    vec![
        (1333, 0),
        (1033, 0),
        (1123, 0),
        (6009, 0),
        (1223, 0),
        (6023, 0),
        (13003, 0),
        (30150, 0),
        (30160, 0),
        (30170, 0),
        (30175, 0),
    ]
}

fn is_encounter_allowed(enc: &UploadEncounterIn) -> bool {
    // Must have a scene id
    let scene_id = match enc.scene_id {
        Some(s) => s,
        None => return false,
    };

    // Never upload overworld encounters
    if scene_id == 5000 {
        return false;
    }

    // Must be in allowed scenes
    let scenes = allowed_scenes_min_hp();
    let min_hp = match scenes.iter().find(|(s, _)| *s == scene_id) {
        Some((_, hp)) => *hp,
        None => return false,
    };

    // Must have at least one detected boss
    if enc.encounter_bosses.is_empty() {
        return false;
    }

    // At least one boss must have max_hp >= min_hp. Treat missing max_hp as 0.
    for b in &enc.encounter_bosses {
        let max_hp = b.max_hp.unwrap_or(0);
        if max_hp >= min_hp {
            return true;
        }
    }
    false
}

#[derive(Clone)]
struct PendingEncounter {
    id: i32,
    payload: UploadEncounterIn,
}

pub async fn perform_upload(
    app: AppHandle,
    api_key: String,
    base_urls: Vec<String>,
) -> Result<(), String> {
    CANCEL_FLAG.store(false, Ordering::SeqCst);

    // BLOCKING: Count total
    let total = tauri::async_runtime::spawn_blocking(|| {
        let path = default_db_path();
        let mut conn = diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy())
            .map_err(|e| e.to_string())?;
        count_ended_encounters(&mut conn)
    })
    .await
    .map_err(|e| e.to_string())??;

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

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    // We process encounters one by one (batch_size = 1).
    // Since we mark encounters as uploaded (removing them from the pending set),
    // we always fetch the first pending encounter (offset 0).
    let batch_size = 1_i64;
    let mut uploaded = 0_i64;
    let mut current_url_index = 0;

    // We loop until we have processed 'total' encounters or run out of pending ones.
    // Note: 'total' is a snapshot at start. New encounters might finish during upload,
    // but we only aim to upload the count we saw at start to avoid infinite loops if
    // encounters are generated faster than upload.
    while uploaded < total {
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

        // BLOCKING: Load batch (always offset 0)
        let (rows, payloads, skipped_policy_ids) =
            tauri::async_runtime::spawn_blocking(move || {
                let path = default_db_path();
                let mut conn = diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy())
                    .map_err(|e| e.to_string())?;
                // Always use offset 0 because uploaded encounters are filtered out by the query
                let rows = load_encounters_slice(&mut conn, 0, batch_size)?;
                if rows.is_empty() {
                    return Ok((rows, Vec::new(), Vec::new()));
                }
                let mut payloads = Vec::with_capacity(rows.len());
                let mut skipped = Vec::new();
                for row in &rows {
                    let built = build_encounter_payload(&mut conn, row.clone())?;
                    if is_encounter_allowed(&built) {
                        payloads.push(PendingEncounter {
                            id: row.0,
                            payload: built,
                        });
                    } else {
                        skipped.push(row.0);
                    }
                }
                Ok::<_, String>((rows, payloads, skipped))
            })
            .await
            .map_err(|e| e.to_string())??;

        if rows.is_empty() {
            break;
        }

        // Notify UI about any local encounters skipped by upload policy
        if !skipped_policy_ids.is_empty() {
            let skipped_payload = json!({"uploaded": uploaded + skipped_policy_ids.len() as i64, "total": total, "skipped_by_policy": skipped_policy_ids.len(), "message": format!("Skipped {} encounter(s) due to upload policy", skipped_policy_ids.len())});
            if let Some(w) = app.get_webview_window(crate::WINDOW_MAIN_LABEL) {
                let _ = w.emit("upload:progress", skipped_payload.clone());
            }
            if let Some(w) = app.get_webview_window(crate::WINDOW_LIVE_LABEL) {
                let _ = w.emit("upload:progress", skipped_payload.clone());
            }
            let _ = app.emit("upload:progress", skipped_payload);
        }

        // Preflight check: ask server which encounters are duplicates
        let hashes: Vec<String> = payloads
            .iter()
            .filter_map(|e| e.payload.source_hash.clone())
            .collect();

        let mut filtered_payloads = payloads.clone();
        let mut duplicate_ids: Vec<i32> = Vec::new();
        let mut duplicate_updates: Vec<(i32, i64)> = Vec::new(); // (local_id, remote_id)

        if !hashes.is_empty() {
            // Perform preflight duplicate check
            let check_url = format!(
                "{}/upload/check",
                base_urls[current_url_index].trim_end_matches('/')
            );
            let check_body = CheckDuplicatesRequest { hashes };

            match client
                .post(&check_url)
                .header("X-Api-Key", api_key.clone())
                .json(&check_body)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(check_result) = resp.json::<CheckDuplicatesResponse>().await {
                        // Map hash -> remote_encounter_id for duplicates
                        let duplicate_map: std::collections::HashMap<String, i64> = check_result
                            .duplicates
                            .iter()
                            .map(|d| (d.hash.clone(), d.encounter_id))
                            .collect();

                        filtered_payloads = payloads
                            .iter()
                            .cloned()
                            .filter(|entry| {
                                if let Some(ref hash) = entry.payload.source_hash {
                                    if let Some(&remote_id) = duplicate_map.get(hash) {
                                        duplicate_ids.push(entry.id);
                                        duplicate_updates.push((entry.id, remote_id));
                                        return false;
                                    }
                                }
                                true // Keep encounters without hash or not duplicates
                            })
                            .collect();

                        let skipped = duplicate_ids.len();
                        if skipped > 0 {
                            // Emit progress for skipped duplicates
                            let progress_payload: serde_json::Value = json!({
                                "uploaded": uploaded + skipped as i64,
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

        // BLOCKING: Update duplicates in DB
        if !duplicate_updates.is_empty() {
            let updates = duplicate_updates.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let path = default_db_path();
                let mut conn = diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy())
                    .map_err(|e| e.to_string())?;
                use sch::encounters::dsl as e;
                for (local_id, remote_id) in updates {
                    let _ = diesel::update(e::encounters.filter(e::id.eq(local_id)))
                        .set((
                            e::remote_encounter_id.eq(remote_id),
                            e::uploaded_at_ms.eq(now_ms()),
                        ))
                        .execute(&mut conn);
                }
                Ok::<_, String>(())
            })
            .await
            .map_err(|e| e.to_string())??;
        }

        // If no payloads remain for upload (either duplicates or filtered by policy), mark
        // duplicates and policy-skipped encounters as uploaded locally and continue.
        if filtered_payloads.is_empty() {
            // mark duplicates + any policy-skipped rows
            let mut ids: Vec<i32> = payloads.iter().map(|p| p.id).collect();
            ids.extend(skipped_policy_ids.iter());

            // BLOCKING: Mark uploaded
            tauri::async_runtime::spawn_blocking(move || {
                let path = default_db_path();
                let mut conn = diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy())
                    .map_err(|e| e.to_string())?;
                mark_encounters_uploaded(&mut conn, &ids)
            })
            .await
            .map_err(|e| e.to_string())??;

            uploaded += rows.len() as i64;
            continue;
        }

        let body = UploadRequestBody {
            encounters: filtered_payloads
                .iter()
                .map(|entry| entry.payload.clone())
                .collect(),
            client_version: Some(env!("APP_VERSION").to_string()),
        };
        let mut processed_ids = duplicate_ids.clone();

        // Try each URL in sequence until one succeeds
        let mut upload_success = false;
        let mut last_error = String::new();

        let mut returned_ids: Vec<i64> = Vec::new();
        while current_url_index < base_urls.len() && !upload_success {
            // Use trailing slash to match server route registration (/api/v1/upload/)
            // Avoids a 307 redirect which can drop custom headers like X-Api-Key.
            let url = format!(
                "{}/upload/",
                base_urls[current_url_index].trim_end_matches('/')
            );
            match client
                .post(&url)
                .header("X-Api-Key", api_key.clone())
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        // Parse response to get created encounter IDs
                        match resp.json::<UploadEncountersResponse>().await {
                            Ok(upload_resp) => {
                                returned_ids = upload_resp.ids;
                                upload_success = true;
                            }
                            Err(e) => {
                                last_error = format!(
                                    "Failed to parse upload response from {}: {}",
                                    base_urls[current_url_index], e
                                );
                                current_url_index += 1;
                            }
                        }
                    } else {
                        let text = resp.text().await.unwrap_or_default();
                        last_error = format!(
                            "Server error from {}: {}",
                            base_urls[current_url_index], text
                        );
                        // Try next URL for server errors
                        current_url_index += 1;
                    }
                }
                Err(e) => {
                    last_error = format!(
                        "Connection error from {}: {}",
                        base_urls[current_url_index], e
                    );
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

        // BLOCKING: Update remote IDs and mark uploaded
        let filtered_ids: Vec<i32> = filtered_payloads.iter().map(|e| e.id).collect();
        let skipped_ids_clone = skipped_policy_ids.clone();

        tauri::async_runtime::spawn_blocking(move || {
            let path = default_db_path();
            let mut conn = diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy())
                .map_err(|e| e.to_string())?;
            use sch::encounters::dsl as e;

            // Map returned remote IDs to local encounter IDs
            if returned_ids.len() == filtered_ids.len() {
                for (local_id, &remote_id) in filtered_ids.iter().zip(returned_ids.iter()) {
                    let _ = diesel::update(e::encounters.filter(e::id.eq(*local_id)))
                        .set((
                            e::remote_encounter_id.eq(remote_id),
                            e::uploaded_at_ms.eq(now_ms()),
                        ))
                        .execute(&mut conn);
                }
            } else {
                log::warn!(
                    "Mismatch between uploaded ({}) and returned ({}) encounter IDs. Remote IDs will not be persisted.",
                    filtered_ids.len(),
                    returned_ids.len()
                );
            }

            // Persist processed ids: duplicates (already present) + uploaded + policy-skipped
            // Note: duplicates were already updated in the previous blocking block, but we need to ensure
            // they are marked uploaded if not already (they should be).
            // Actually, duplicates were updated with uploaded_at_ms in the previous block.
            // So we only need to handle filtered_ids (which we just updated above) and skipped_policy_ids.

            // We need to mark skipped_policy_ids as uploaded.
            // And filtered_ids were marked uploaded in the loop above.
            // So we just need to mark skipped_policy_ids.

            mark_encounters_uploaded(&mut conn, &skipped_ids_clone)?;

            Ok::<_, String>(())
        })
        .await
        .map_err(|e| e.to_string())??;

        uploaded += rows.len() as i64;
        // Emit progress for UI to known windows and as a fallback via app.emit
        let progress_payload: serde_json::Value = json!({"uploaded": uploaded, "total": total});
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

pub fn cancel_upload() {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
}

#[tauri::command]
#[specta::specta]
pub async fn start_upload(
    app: tauri::AppHandle,
    api_key: String,
    base_url: Option<String>,
) -> Result<(), String> {
    let key = api_key.trim().to_string();
    if key.is_empty() {
        return Err("API key is required".to_string());
    }

    let base_urls = resolve_base_urls(base_url);
    let guard = UploadRunGuard::try_acquire()
        .ok_or_else(|| "An upload is already in progress".to_string())?;

    let app_cloned = app.clone();
    tauri::async_runtime::spawn(async move {
        let _guard = guard;
        if let Err(e) = perform_upload(app_cloned.clone(), key, base_urls).await {
            // best-effort error event
            let _ = app_cloned.emit("upload:error", json!({"message": e}));
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
        // Test that when no base_url is provided, it uses the public API first
        // and then falls back to the local development server.
        let base_urls = resolve_base_urls(None);

        // Verify the ordering
        assert_eq!(base_urls.len(), 2);
        assert_eq!(base_urls[0], "https://api.bpsr.app/api/v1");
        assert!(base_urls[1].contains("localhost") || base_urls[1].contains("127.0.0.1"));
    }

    #[test]
    fn test_specific_base_url_only() {
        // Test that when a specific base_url is provided, it is ignored and the
        // fixed ordering (public API then localhost) is still used.
        let specific_url = "https://custom.example.com".to_string();
        let base_urls = resolve_base_urls(Some(specific_url));

        assert_eq!(base_urls.len(), 2);
        assert_eq!(base_urls[0], "https://api.bpsr.app/api/v1");
    }

    #[test]
    fn test_url_formatting() {
        // Test that URLs are formatted correctly when constructing the upload endpoint
        let base_urls = vec![
            "http://localhost:8080/api/v1/".to_string(),
            "https://api.bpsr.app/api/v1".to_string(),
            "https://custom.example.com/api/".to_string(),
        ];

        let expected_endpoints = vec![
            "http://localhost:8080/api/v1/upload/",
            "https://api.bpsr.app/api/v1/upload/",
            "https://custom.example.com/api/upload/",
        ];

        for (i, base_url) in base_urls.iter().enumerate() {
            let formatted_url = format!("{}/upload/", base_url.trim_end_matches('/'));
            assert_eq!(formatted_url, expected_endpoints[i]);
        }
    }

    #[test]
    fn test_upload_request_includes_client_version() {
        let body = UploadRequestBody {
            encounters: vec![],
            client_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        };
        let v = serde_json::to_value(&body).expect("serialize");
        assert!(v.get("clientVersion").is_some());
    }

    #[test]
    fn blocks_overworld_scene_from_upload() {
        let encounter = UploadEncounterIn {
            started_at_ms: 0,
            ended_at_ms: Some(1),
            local_player_id: None,
            total_dmg: Some(0),
            total_heal: Some(0),
            scene_id: Some(5000),
            scene_name: Some("Overworld".to_string()),
            source_hash: None,
            attempts: vec![],
            death_events: vec![],
            actor_encounter_stats: vec![],
            damage_skill_stats: vec![],
            heal_skill_stats: vec![],
            entities: vec![],
            encounter_bosses: vec![UploadEncounterBossIn {
                monster_name: "Dummy".to_string(),
                hits: 0,
                total_damage: 0,
                max_hp: Some(1_000),
                is_defeated: false,
            }],
            detailed_playerdata: vec![],
            dungeon_segments: vec![],
        };

        assert!(!is_encounter_allowed(&encounter));
    }
}
