use crate::module_extractor::types::{ImportModulesResponse, UnknownAttribute};
/// Tauri commands for module sync functionality
/// Exposes module extraction and upload to the frontend
use crate::module_extractor::{extract_modules, upload_modules};
use crate::uploader::AutoUploadState;
use crate::uploader::player_data_sync::PlayerDataSyncState;
use blueprotobuf_lib::blueprotobuf::SyncContainerData;
use log::{error, info};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Failed upload queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedUpload {
    pub modules: Vec<crate::module_extractor::types::ModuleInfo>,
    pub timestamp: String,
    pub retry_count: u32,
    pub last_error: String,
}

const LEGACY_LOCAL_BASE_URL: &str = "http://localhost:8080/api/v1";

fn default_api_base_url() -> String {
    std::env::var("WEBSITE_API_BASE").unwrap_or_else(|_| "https://api.bpsr.app/api/v1".to_string())
}

/// Module sync state shared across commands
#[derive(Clone)]
pub struct ModuleSyncState {
    /// Last extracted modules (cached for manual sync)
    pub last_modules: Arc<RwLock<Vec<crate::module_extractor::types::ModuleInfo>>>,
    /// Module sync enabled flag
    pub sync_enabled: Arc<RwLock<bool>>,
    /// API key for authentication
    pub api_key: Arc<RwLock<Option<String>>>,
    /// Base URL for API
    pub base_url: Arc<RwLock<String>>,
    /// Hash of last uploaded module set (for deduplication)
    pub last_upload_hash: Arc<RwLock<Option<String>>>,
    /// Set of module UUIDs from last sync (for differential sync)
    pub last_sync_uuids: Arc<RwLock<HashSet<String>>>,
    /// Auto-sync interval in minutes (0 = disabled)
    pub auto_sync_interval_minutes: Arc<RwLock<u32>>,
    /// Failed upload queue
    pub failed_uploads: Arc<RwLock<Vec<FailedUpload>>>,
    /// Unknown attribute tracking
    pub unknown_attributes: Arc<RwLock<HashMap<i32, UnknownAttribute>>>,
}

impl Default for ModuleSyncState {
    fn default() -> Self {
        Self {
            last_modules: Arc::new(RwLock::new(Vec::new())),
            sync_enabled: Arc::new(RwLock::new(false)),
            api_key: Arc::new(RwLock::new(None)),
            base_url: Arc::new(RwLock::new(default_api_base_url())),
            last_upload_hash: Arc::new(RwLock::new(None)),
            last_sync_uuids: Arc::new(RwLock::new(HashSet::new())),
            auto_sync_interval_minutes: Arc::new(RwLock::new(0)),
            failed_uploads: Arc::new(RwLock::new(Vec::new())),
            unknown_attributes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Calculate hash of module set for deduplication
fn calculate_module_set_hash(modules: &[crate::module_extractor::types::ModuleInfo]) -> String {
    let mut hasher = Sha256::new();

    // Sort modules by UUID for consistent hashing
    let mut sorted_modules = modules.to_vec();
    sorted_modules.sort_by(|a, b| a.uuid.cmp(&b.uuid));

    for module in sorted_modules {
        hasher.update(module.content_hash().as_bytes());
    }

    hex::encode(hasher.finalize())
}

/// Response for module sync status query
#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct ModuleSyncStatus {
    pub enabled: bool,
    pub has_api_key: bool,
    pub last_module_count: usize,
    pub base_url: String,
    pub auto_sync_interval_minutes: u32,
    pub failed_uploads_count: usize,
    pub unknown_attributes_count: usize,
}

/// Set module sync configuration
#[tauri::command]
#[specta::specta]
pub async fn set_module_sync_config(
    state: tauri::State<'_, ModuleSyncState>,
    auto_upload_state: tauri::State<'_, AutoUploadState>,
    player_data_sync_state: tauri::State<'_, PlayerDataSyncState>,
    enabled: bool,
    api_key: Option<String>,
    base_url: Option<String>,
    auto_sync_interval_minutes: Option<u32>,
) -> Result<(), String> {
    *state.sync_enabled.write().await = enabled;

    {
        let mut api_key_guard = state.api_key.write().await;
        match api_key {
            Some(key) => {
                let trimmed = key.trim().to_string();
                if trimmed.is_empty() {
                    *api_key_guard = None;
                } else {
                    *api_key_guard = Some(trimmed);
                }
            }
            None => {
                *api_key_guard = None;
            }
        }
    }

    if let Some(url) = base_url {
        let trimmed = url.trim().to_string();
        let resolved = if trimmed.is_empty() || trimmed == LEGACY_LOCAL_BASE_URL {
            default_api_base_url()
        } else {
            trimmed
        };
        *state.base_url.write().await = resolved;
    }

    if let Some(interval) = auto_sync_interval_minutes {
        *state.auto_sync_interval_minutes.write().await = interval;
    }

    let current_key = state.api_key.read().await.clone();
    let current_base = state.base_url.read().await.clone();
    auto_upload_state
        .sync_from_settings(current_key.clone(), Some(current_base.clone()))
        .await;
    // Also sync player data sync state with the same settings
    player_data_sync_state
        .sync_from_settings(current_key, Some(current_base))
        .await;

    info!("Module sync config updated: enabled={}", enabled);
    Ok(())
}

/// Get current module sync status
#[tauri::command]
#[specta::specta]
pub fn get_module_sync_status(
    state: tauri::State<'_, ModuleSyncState>,
) -> Result<ModuleSyncStatus, String> {
    let enabled = *state.sync_enabled.blocking_read();
    let has_api_key = state.api_key.blocking_read().is_some();
    let last_module_count = state.last_modules.blocking_read().len();
    let base_url = state.base_url.blocking_read().clone();
    let auto_sync_interval_minutes = *state.auto_sync_interval_minutes.blocking_read();
    let failed_uploads_count = state.failed_uploads.blocking_read().len();
    let unknown_attributes_count = state.unknown_attributes.blocking_read().len();

    Ok(ModuleSyncStatus {
        enabled,
        has_api_key,
        last_module_count,
        base_url,
        auto_sync_interval_minutes,
        failed_uploads_count,
        unknown_attributes_count,
    })
}

/// Manually trigger module sync (uploads last extracted modules)
#[tauri::command]
#[specta::specta]
pub fn trigger_module_sync(
    state: tauri::State<'_, ModuleSyncState>,
) -> Result<ImportModulesResponse, String> {
    let enabled = *state.sync_enabled.blocking_read();
    if !enabled {
        return Err("Module sync is disabled".to_string());
    }

    let api_key = match state.api_key.blocking_read().clone() {
        Some(key) => key,
        None => return Err("No API key configured".to_string()),
    };

    let base_url = state.base_url.blocking_read().clone();
    let modules = state.last_modules.blocking_read().clone();

    if modules.is_empty() {
        return Err("No modules to sync (capture module data first)".to_string());
    }

    // Check deduplication - don't upload if hash matches
    let current_hash = calculate_module_set_hash(&modules);
    if let Some(last_hash) = state.last_upload_hash.blocking_read().as_ref() {
        if &current_hash == last_hash {
            info!("Module set unchanged (hash match), skipping upload");
            return Ok(ImportModulesResponse {
                summary: crate::module_extractor::types::ImportSummary {
                    added: 0,
                    updated: 0,
                    errors: 0,
                },
                errors: vec![],
            });
        }
    }

    info!(
        "Manually triggering module sync for {} modules",
        modules.len()
    );

    let rt = tokio::runtime::Handle::current();
    let response = rt.block_on(upload_modules(modules, &api_key, &base_url))?;

    // Update last upload hash on success
    *state.last_upload_hash.blocking_write() = Some(current_hash);

    Ok(response)
}

/// Retry failed uploads
#[tauri::command]
#[specta::specta]
pub async fn retry_failed_uploads(
    state: tauri::State<'_, ModuleSyncState>,
) -> Result<Vec<ImportModulesResponse>, String> {
    let api_key = match state.api_key.read().await.clone() {
        Some(key) => key,
        None => return Err("No API key configured".to_string()),
    };

    let base_url = state.base_url.read().await.clone();
    let mut failed_uploads = state.failed_uploads.write().await;

    let mut responses = Vec::new();
    let mut successful_indices = Vec::new();

    for (idx, failed) in failed_uploads.iter().enumerate() {
        match upload_modules(failed.modules.clone(), &api_key, &base_url).await {
            Ok(response) => {
                info!("Retry successful for upload from {}", failed.timestamp);
                responses.push(response);
                successful_indices.push(idx);
            }
            Err(e) => {
                error!("Retry failed: {}", e);
            }
        }
    }

    // Remove successful uploads from queue (in reverse to maintain indices)
    for idx in successful_indices.into_iter().rev() {
        failed_uploads.remove(idx);
    }

    Ok(responses)
}

/// Get unknown attributes for telemetry
#[tauri::command]
#[specta::specta]
pub fn get_unknown_attributes(
    state: tauri::State<'_, ModuleSyncState>,
) -> Result<Vec<UnknownAttribute>, String> {
    let unknown_attrs = state.unknown_attributes.blocking_read();
    Ok(unknown_attrs.values().cloned().collect())
}

/// Clear unknown attributes tracking
#[tauri::command]
#[specta::specta]
pub async fn clear_unknown_attributes(
    state: tauri::State<'_, ModuleSyncState>,
) -> Result<(), String> {
    state.unknown_attributes.write().await.clear();
    Ok(())
}

/// Internal function to process SyncContainerData and extract modules
/// Called by the state manager when SyncContainerData is received
pub async fn process_sync_container_data(
    state: &ModuleSyncState,
    sync_data: &SyncContainerData,
    auto_upload: bool,
) -> Result<usize, String> {
    // Extract modules
    let (modules, unknown_part_ids) = extract_modules_with_tracking(sync_data);
    let module_count = modules.len();

    // Track unknown attributes
    if !unknown_part_ids.is_empty() {
        let mut unknown_attrs = state.unknown_attributes.write().await;
        let now = chrono::Utc::now().to_rfc3339();

        for (part_id, config_ids) in unknown_part_ids {
            unknown_attrs
                .entry(part_id)
                .and_modify(|attr| {
                    attr.occurrence_count += 1;
                    for config_id in &config_ids {
                        if !attr.module_config_ids.contains(config_id) {
                            attr.module_config_ids.push(*config_id);
                        }
                    }
                })
                .or_insert(UnknownAttribute {
                    part_id,
                    first_seen: now.clone(),
                    occurrence_count: 1,
                    module_config_ids: config_ids,
                });
        }
    }

    // Differential sync - filter changed modules
    let changed_modules = if auto_upload {
        let last_uuids = state.last_sync_uuids.read().await;
        modules
            .iter()
            .filter(|m| !last_uuids.contains(&m.uuid))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        modules.clone()
    };

    // Update cached modules
    *state.last_modules.write().await = modules.clone();

    // Auto-upload if enabled and configured
    if auto_upload && !changed_modules.is_empty() {
        let enabled = *state.sync_enabled.read().await;
        if !enabled {
            return Ok(module_count);
        }

        let api_key = match state.api_key.read().await.clone() {
            Some(key) => key,
            None => {
                error!("Module sync enabled but no API key configured");
                return Ok(module_count);
            }
        };

        // Check deduplication
        let current_hash = calculate_module_set_hash(&changed_modules);
        let should_upload = match state.last_upload_hash.read().await.as_ref() {
            Some(last_hash) => &current_hash != last_hash,
            None => true,
        };

        if !should_upload {
            info!("Module set unchanged (hash match), skipping auto-upload");
            return Ok(module_count);
        }

        let base_url = state.base_url.read().await.clone();
        let state_clone = state.clone();

        // Upload in background (don't block)
        let modules_clone = changed_modules.clone();
        tokio::spawn(async move {
            match upload_modules(modules_clone.clone(), &api_key, &base_url).await {
                Ok(response) => {
                    info!(
                        "Auto-sync modules: added={}, updated={}, errors={}",
                        response.summary.added, response.summary.updated, response.summary.errors
                    );

                    // Update last upload hash and sync UUIDs
                    *state_clone.last_upload_hash.write().await = Some(current_hash);
                    let mut last_uuids = state_clone.last_sync_uuids.write().await;
                    for module in &modules_clone {
                        last_uuids.insert(module.uuid.clone());
                    }
                }
                Err(e) => {
                    error!("Auto-sync modules failed: {}", e);

                    // Add to failed uploads queue
                    let mut failed_uploads = state_clone.failed_uploads.write().await;
                    failed_uploads.push(FailedUpload {
                        modules: modules_clone,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        retry_count: 0,
                        last_error: e,
                    });

                    // Limit queue size
                    if failed_uploads.len() > 10 {
                        failed_uploads.remove(0);
                    }
                }
            }
        });
    }

    Ok(module_count)
}

/// Extract modules with unknown attribute tracking
fn extract_modules_with_tracking(
    sync_data: &SyncContainerData,
) -> (
    Vec<crate::module_extractor::types::ModuleInfo>,
    HashMap<i32, Vec<i32>>,
) {
    let modules = extract_modules(sync_data);
    let mut unknown_part_ids: HashMap<i32, Vec<i32>> = HashMap::new();

    // Track unknown attributes
    for module in &modules {
        for part in &module.parts {
            if part.name.starts_with("未知属性") {
                unknown_part_ids
                    .entry(part.part_id)
                    .or_insert_with(Vec::new)
                    .push(module.config_id);
            }
        }
    }

    (modules, unknown_part_ids)
}

/// Start background auto-sync timer (called on app startup)
pub async fn start_auto_sync_timer(state: ModuleSyncState) {
    tokio::spawn(async move {
        let mut minutes_since_last_sync = 0u32;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

            let interval = *state.auto_sync_interval_minutes.read().await;
            if interval == 0 {
                continue;
            }

            let enabled = *state.sync_enabled.read().await;
            if !enabled {
                continue;
            }

            // Check if it's time to sync
            minutes_since_last_sync += 1;
            if minutes_since_last_sync >= interval {
                minutes_since_last_sync = 0;

                // Trigger sync
                info!("Auto-sync timer triggered (interval: {} minutes)", interval);

                let api_key = match state.api_key.read().await.clone() {
                    Some(key) => key,
                    None => continue,
                };

                let base_url = state.base_url.read().await.clone();
                let modules = state.last_modules.read().await.clone();

                if !modules.is_empty() {
                    let state_clone = state.clone();
                    tokio::spawn(async move {
                        match upload_modules(modules, &api_key, &base_url).await {
                            Ok(response) => {
                                info!(
                                    "Scheduled auto-sync succeeded: added={}, updated={}",
                                    response.summary.added, response.summary.updated
                                );
                            }
                            Err(e) => {
                                error!("Scheduled auto-sync failed: {}", e);
                            }
                        }
                    });
                }
            }
        }
    });
}
