use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use reqwest::Client;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Notify, RwLock};
use tokio::time::{Duration, interval};

use crate::database::schema as sch;
use crate::database::{establish_connection, now_ms};
use diesel::prelude::*;

use super::{get_module_sync_settings_path, read_module_sync_settings_blocking};

static PLAYER_DATA_SYNC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Sync interval for player data (15 minutes)
const PLAYER_DATA_SYNC_INTERVAL_SECS: u64 = 15 * 60;

/// State for the automatic player data sync task
#[derive(Clone, Default)]
pub struct PlayerDataSyncState {
    api_key: Arc<RwLock<Option<String>>>,
    base_url: Arc<RwLock<Option<String>>>,
    auto_upload_enabled: Arc<RwLock<bool>>,
    notifier: Arc<Notify>,
    /// Tracks the last sync time to ensure we only sync data updated since then
    last_sync_ms: Arc<RwLock<i64>>,
}

impl PlayerDataSyncState {
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
            None => return,
        };

        // Read file in a blocking thread to avoid blocking the async runtime
        let settings = match tauri::async_runtime::spawn_blocking(move || {
            read_module_sync_settings_blocking(settings_path)
        })
        .await
        {
            Ok(s) => s,
            Err(_) => return,
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
            *self.api_key.write().await = api_key;
            *self.base_url.write().await = base_url;
            *self.auto_upload_enabled.write().await = settings.auto_upload;
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

    pub async fn get_last_sync_ms(&self) -> i64 {
        *self.last_sync_ms.read().await
    }

    pub async fn set_last_sync_ms(&self, ms: i64) {
        *self.last_sync_ms.write().await = ms;
    }
}

struct PlayerDataSyncGuard;

impl PlayerDataSyncGuard {
    fn try_acquire() -> Option<Self> {
        if PLAYER_DATA_SYNC_IN_PROGRESS
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            Some(Self)
        } else {
            None
        }
    }
}

impl Drop for PlayerDataSyncGuard {
    fn drop(&mut self) {
        PLAYER_DATA_SYNC_IN_PROGRESS.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncPlayerDataIn {
    pub player_id: i64,
    pub last_seen_ms: i64,
    pub char_serialize_json: String,
    pub profession_list_json: Option<String>,
    pub talent_node_ids_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPlayerDataRequest {
    pub player_data: Vec<SyncPlayerDataIn>,
    pub client_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPlayerDataResponse {
    pub synced: i32,
    pub updated: i32,
    pub created: i32,
    pub ids: Vec<i64>,
}

fn resolve_base_urls(_base_url: Option<String>) -> Vec<String> {
    // Same logic as encounter uploader - use fixed primary+fallback
    vec![
        "https://api.bpsr.app/api/v1".to_string(),
        "http://localhost:8080/api/v1".to_string(),
    ]
}

/// Load all detailed player data from the local database
fn load_all_player_data(
    conn: &mut diesel::sqlite::SqliteConnection,
) -> Result<Vec<SyncPlayerDataIn>, String> {
    use sch::detailed_playerdata::dsl as dpd;

    let rows = dpd::detailed_playerdata
        .select((
            dpd::player_id,
            dpd::last_seen_ms,
            dpd::char_serialize_json,
            dpd::profession_list_json,
            dpd::talent_node_ids_json,
        ))
        .load::<(i64, i64, String, Option<String>, Option<String>)>(conn)
        .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(
            |(
                player_id,
                last_seen_ms,
                char_serialize_json,
                profession_list_json,
                talent_node_ids_json,
            )| {
                SyncPlayerDataIn {
                    player_id,
                    last_seen_ms,
                    char_serialize_json,
                    profession_list_json,
                    talent_node_ids_json,
                }
            },
        )
        .collect())
}

/// Count player data entries
fn count_player_data() -> Result<i64, String> {
    let mut conn = establish_connection().map_err(|e| e.to_string())?;

    use sch::detailed_playerdata::dsl as dpd;
    dpd::detailed_playerdata
        .count()
        .get_result(&mut conn)
        .map_err(|e| e.to_string())
}

/// Start the automatic player data sync background task
pub fn start_player_data_sync_task(app: AppHandle, state: PlayerDataSyncState) {
    tauri::async_runtime::spawn(async move {
        let task_span = tracing::info_span!(target: "app::sync", "player_data_sync_task");
        let _task_guard = task_span.enter();

        // Load settings once at startup
        state.reload_from_file(&app).await;

        let mut ticker = interval(Duration::from_secs(PLAYER_DATA_SYNC_INTERVAL_SECS));
        let notifier = state.notifier();

        loop {
            tokio::select! {
                _ = ticker.tick() => {},
                _ = notifier.notified() => {
                    // Settings changed notification - reload settings
                    state.reload_from_file(&app).await;
                },
            }

            if let Err(err) = maybe_trigger_player_data_sync(app.clone(), state.clone()).await {
                log::warn!(target: "app::sync", "player data sync check failed: {}", err);
            }
        }
    });
}

async fn maybe_trigger_player_data_sync(
    app: AppHandle,
    state: PlayerDataSyncState,
) -> Result<(), String> {
    // Check if auto-upload is enabled (uses cached value)
    if !state.is_auto_upload_enabled().await {
        return Ok(());
    }

    let api_key = match state.current_api_key().await {
        Some(key) => key,
        None => return Ok(()), // No API key configured, skip sync
    };

    let pending = tauri::async_runtime::spawn_blocking(|| count_player_data())
        .await
        .map_err(|e| e.to_string())??;

    if pending == 0 {
        return Ok(()); // No player data to sync
    }

    log::info!(target: "app::sync", "player data sync: pending={} starting", pending);

    if let Some(guard) = PlayerDataSyncGuard::try_acquire() {
        let base_urls = resolve_base_urls(state.current_base_url().await);
        let app_clone = app.clone();
        let state_clone = state.clone();

        tauri::async_runtime::spawn(async move {
            let _guard = guard;
            if let Err(e) =
                perform_player_data_sync(app_clone.clone(), api_key, base_urls, state_clone).await
            {
                let _ = app_clone.emit("player-data-sync:error", json!({"message": e}));
            }
        });
    }

    Ok(())
}

/// Perform the player data sync to the server
pub async fn perform_player_data_sync(
    app: AppHandle,
    api_key: String,
    base_urls: Vec<String>,
    state: PlayerDataSyncState,
) -> Result<(), String> {
    let started = std::time::Instant::now();
    let sync_span = tracing::info_span!(target: "app::sync", "player_data_sync_run", base_urls = base_urls.len());
    let _sync_guard = sync_span.enter();

    let mut conn = establish_connection().map_err(|e| e.to_string())?;

    let player_data = load_all_player_data(&mut conn)?;

    if player_data.is_empty() {
        let _ = app.emit(
            "player-data-sync:completed",
            json!({"synced": 0, "total": 0}),
        );
        log::info!(target: "app::sync", "player data sync done total=0 elapsed_ms={}", started.elapsed().as_millis());
        return Ok(());
    }

    let total = player_data.len();
    log::info!(target: "app::sync", "player data sync started total={}", total);

    // Emit started event
    let started_payload = json!({"total": total});
    if let Some(w) = app.get_webview_window(crate::WINDOW_MAIN_LABEL) {
        let _ = w.emit("player-data-sync:started", started_payload.clone());
    }
    let _ = app.emit("player-data-sync:started", started_payload);

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let body = SyncPlayerDataRequest {
        player_data,
        client_version: Some(env!("APP_VERSION").to_string()),
    };

    let mut current_url_index = 0;
    let mut sync_success = false;
    let mut last_error = String::new();
    let mut response_data: Option<SyncPlayerDataResponse> = None;

    while current_url_index < base_urls.len() && !sync_success {
        let url = format!(
            "{}/upload/player-data",
            base_urls[current_url_index].trim_end_matches('/')
        );

        match client
            .post(&url)
            .header("X-Api-Key", api_key.clone())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<SyncPlayerDataResponse>().await {
                        Ok(sync_resp) => {
                            response_data = Some(sync_resp);
                            sync_success = true;
                        }
                        Err(e) => {
                            last_error = format!(
                                "Failed to parse player data sync response from {}: {}",
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
                    current_url_index += 1;
                }
            }
            Err(e) => {
                last_error = format!(
                    "Connection error to {}: {}",
                    base_urls[current_url_index], e
                );
                current_url_index += 1;
            }
        }
    }

    if !sync_success {
        let err_payload = json!({"message": last_error});
        if let Some(w) = app.get_webview_window(crate::WINDOW_MAIN_LABEL) {
            let _ = w.emit("player-data-sync:error", err_payload.clone());
        }
        let _ = app.emit("player-data-sync:error", err_payload);
        log::warn!(target: "app::sync", "player data sync failed total={} elapsed_ms={} err={}", total, started.elapsed().as_millis(), last_error);
        return Err(last_error);
    }

    // Update last sync time
    state.set_last_sync_ms(now_ms()).await;

    // Emit completed event
    let completed_payload = if let Some(ref resp) = response_data {
        json!({
            "synced": resp.synced,
            "updated": resp.updated,
            "created": resp.created,
            "total": total
        })
    } else {
        json!({"synced": total, "total": total})
    };

    if let Some(w) = app.get_webview_window(crate::WINDOW_MAIN_LABEL) {
        let _ = w.emit("player-data-sync:completed", completed_payload.clone());
    }
    let _ = app.emit("player-data-sync:completed", completed_payload);

    log::info!(
        target: "app::sync",
        "player data sync completed synced={} total={} elapsed_ms={}",
        response_data.as_ref().map(|r| r.synced).unwrap_or(0),
        total,
        started.elapsed().as_millis()
    );

    Ok(())
}

/// Manual trigger for player data sync (can be called from Tauri command)
#[tauri::command]
#[specta::specta]
pub async fn sync_player_data(
    app: tauri::AppHandle,
    api_key: String,
    base_url: Option<String>,
) -> Result<(), String> {
    let key = api_key.trim().to_string();
    if key.is_empty() {
        return Err("API key is required".to_string());
    }

    let base_urls = resolve_base_urls(base_url);
    let guard = PlayerDataSyncGuard::try_acquire()
        .ok_or_else(|| "A player data sync is already in progress".to_string())?;

    let app_cloned = app.clone();
    let state = (*app.state::<PlayerDataSyncState>()).clone();

    tauri::async_runtime::spawn(async move {
        let _guard = guard;
        if let Err(e) = perform_player_data_sync(app_cloned.clone(), key, base_urls, state).await {
            let _ = app_cloned.emit("player-data-sync:error", json!({"message": e}));
        }
    });

    Ok(())
}

/// Response for player data times query
#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDataTimesResponse {
    pub last_detected_ms: Option<i64>,
    pub last_sync_ms: Option<i64>,
}

/// Get the most recent player data detection and sync times
#[tauri::command]
#[specta::specta]
pub async fn get_player_data_times() -> Result<PlayerDataTimesResponse, String> {
    // Get last detected time from the database (the most recent last_seen_ms in detailed_playerdata)
    let last_detected_ms = tauri::async_runtime::spawn_blocking(|| {
        let mut conn = establish_connection().map_err(|e| e.to_string())?;

        use sch::detailed_playerdata::dsl as dpd;
        let result: Result<Option<i64>, diesel::result::Error> = dpd::detailed_playerdata
            .select(diesel::dsl::max(dpd::last_seen_ms))
            .first(&mut conn);

        match result {
            Ok(ms) => Ok(ms),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    // Note: last_sync_ms would need to be stored somewhere persistent
    // For now, we return None as it's tracked in-memory by the frontend
    Ok(PlayerDataTimesResponse {
        last_detected_ms,
        last_sync_ms: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_url_ordering() {
        let base_urls = resolve_base_urls(None);
        assert_eq!(base_urls.len(), 2);
        assert_eq!(base_urls[0], "https://api.bpsr.app/api/v1");
        assert!(base_urls[1].contains("localhost"));
    }
}
