use crate::WINDOW_LIVE_LABEL;
use crate::live::dungeon_log::{self, DungeonLogRuntime};
use crate::live::state::{AppStateManager, StateEvent};
use log::info;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use window_vibrancy::{apply_blur, clear_blur};
// request_restart is not needed in this module at present
use crate::live::event_manager; // for generate_skills_window_*

/// Prettifies a player's name.
///
/// # Arguments
///
/// * `player_uid` - The UID of the player.
/// * `local_player_uid` - The UID of the local player.
/// * `player_name` - The name of the player.
///
/// # Returns
///
/// * `String` - The prettified name.
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

/// Returns 0.0 if the value is NaN or infinite.
///
/// # Arguments
///
/// * `value` - The value to check.
///
/// # Returns
///
/// * `f64` - 0.0 if the value is NaN or infinite, otherwise the value.
fn nan_is_zero(value: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        0.0
    } else {
        value
    }
}

/// Subscribes to a player's skills.
///
/// # Arguments
///
/// * `uid` - The UID of the player.
/// * `skill_type` - The type of skill to subscribe to.
/// * `state_manager` - The state manager.
///
/// # Returns
///
/// * `Result<crate::live::commands_models::SkillsWindow, String>` - The skills window.
#[tauri::command]
#[specta::specta]
pub async fn subscribe_player_skills(
    uid: i64,
    skill_type: String,
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<crate::live::commands_models::SkillsWindow, String> {
    // Register subscription
    state_manager
        .update_skill_subscriptions(|subs| {
            subs.insert((uid, skill_type.clone()));
        })
        .await;

    // Compute and return initial window directly from Encounter
    state_manager
        .with_state(|state| {
            let boss_only = state.boss_only_dps;
            let segment_elapsed_ms = if state.dungeon_segments_enabled {
                dungeon_log::snapshot(&state.dungeon_log).and_then(|log| {
                    log.segments
                        .iter()
                        .rev()
                        .find(|s| s.ended_at_ms.is_none())
                        .map(|segment| {
                            let start_ms = segment.started_at_ms.max(0) as u128;
                            let end_ms = segment
                                .ended_at_ms
                                .map(|t| t.max(0) as u128)
                                .unwrap_or(state.encounter.time_last_combat_packet_ms);
                            end_ms.saturating_sub(start_ms)
                        })
                })
            } else {
                None
            };

            match skill_type.as_str() {
                "dps" => event_manager::generate_skills_window_dps(
                    &state.encounter,
                    uid,
                    boss_only,
                    segment_elapsed_ms,
                )
                .ok_or_else(|| format!("No DPS skills found for player {}", uid)),
                "heal" => event_manager::generate_skills_window_heal(
                    &state.encounter,
                    uid,
                    segment_elapsed_ms,
                )
                .ok_or_else(|| format!("No heal skills found for player {}", uid)),
                "tanked" => event_manager::generate_skills_window_tanked(
                    &state.encounter,
                    uid,
                    segment_elapsed_ms,
                )
                .ok_or_else(|| format!("No tanked skills found for player {}", uid)),
                _ => Err(format!("Invalid skill type: {}", skill_type)),
            }
        })
        .await
}

/// Unsubscribes from a player's skills.
///
/// # Arguments
///
/// * `uid` - The UID of the player.
/// * `skill_type` - The type of skill to unsubscribe from.
/// * `state_manager` - The state manager.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result.
#[tauri::command]
#[specta::specta]
pub async fn unsubscribe_player_skills(
    uid: i64,
    skill_type: String,
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<(), String> {
    state_manager
        .update_skill_subscriptions(|subs| {
            subs.remove(&(uid, skill_type));
        })
        .await;
    Ok(())
}

/// Gets a player's skills.
///
/// # Arguments
///
/// * `uid` - The UID of the player.
/// * `skill_type` - The type of skill to get.
/// * `state_manager` - The state manager.
///
/// # Returns
///
/// * `Result<crate::live::commands_models::SkillsWindow, String>` - The skills window.
#[tauri::command]
#[specta::specta]
pub async fn get_player_skills(
    uid: i64,
    skill_type: String,
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<crate::live::commands_models::SkillsWindow, String> {
    state_manager
        .with_state(|state| {
            let boss_only = state.boss_only_dps;
            let segment_elapsed_ms = if state.dungeon_segments_enabled {
                dungeon_log::snapshot(&state.dungeon_log).and_then(|log| {
                    log.segments
                        .iter()
                        .rev()
                        .find(|s| s.ended_at_ms.is_none())
                        .map(|segment| {
                            let start_ms = segment.started_at_ms.max(0) as u128;
                            let end_ms = segment
                                .ended_at_ms
                                .map(|t| t.max(0) as u128)
                                .unwrap_or(state.encounter.time_last_combat_packet_ms);
                            end_ms.saturating_sub(start_ms)
                        })
                })
            } else {
                None
            };
            match skill_type.as_str() {
                "dps" => event_manager::generate_skills_window_dps(
                    &state.encounter,
                    uid,
                    boss_only,
                    segment_elapsed_ms,
                )
                .ok_or_else(|| format!("No DPS skills found for player {}", uid)),
                "heal" => event_manager::generate_skills_window_heal(
                    &state.encounter,
                    uid,
                    segment_elapsed_ms,
                )
                .ok_or_else(|| format!("No heal skills found for player {}", uid)),
                "tanked" => event_manager::generate_skills_window_tanked(
                    &state.encounter,
                    uid,
                    segment_elapsed_ms,
                )
                .ok_or_else(|| format!("No tanked skills found for player {}", uid)),
                _ => Err(format!("Invalid skill type: {}", skill_type)),
            }
        })
        .await
}

/// Sets whether to only show boss DPS.
///
/// # Arguments
///
/// * `enabled` - Whether to enable boss-only DPS.
/// * `state_manager` - The state manager.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result.
#[tauri::command]
#[specta::specta]
pub async fn set_boss_only_dps(
    enabled: bool,
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<(), String> {
    state_manager
        .with_state_mut(|state| {
            state.boss_only_dps = enabled;
        })
        .await;
    // Recompute and emit updates immediately
    state_manager.update_and_emit_events().await;
    Ok(())
}

/// Enables or disables dungeon segment tracking.
#[tauri::command]
#[specta::specta]
pub async fn set_dungeon_segments_enabled(
    enabled: bool,
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<(), String> {
    let runtime = state_manager
        .with_state_mut(|state| {
            state.dungeon_segments_enabled = enabled;
            DungeonLogRuntime::new(state.dungeon_log.clone(), state.app_handle.clone())
        })
        .await;

    let snapshot = runtime.snapshot();
    dungeon_log::emit_if_changed(&runtime.app_handle, snapshot);
    Ok(())
}

/// Returns the current dungeon log snapshot for the frontend.
#[tauri::command]
#[specta::specta]
pub async fn get_dungeon_log(
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<dungeon_log::DungeonLog, String> {
    let shared_log = state_manager
        .with_state(|state| state.dungeon_log.clone())
        .await;

    dungeon_log::snapshot(&shared_log).ok_or_else(|| "Failed to read dungeon log state".to_string())
}

/// Enables blur on the live meter window.
///
/// # Arguments
///
/// * `app` - A handle to the Tauri application instance.
#[tauri::command]
#[specta::specta]
pub fn enable_blur(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(WINDOW_LIVE_LABEL) {
        apply_blur(&meter_window, Some((10, 10, 10, 50))).ok();
    }
}

/// Disables blur on the live meter window.
///
/// # Arguments
///
/// * `app` - A handle to the Tauri application instance.
#[tauri::command]
#[specta::specta]
pub fn disable_blur(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(WINDOW_LIVE_LABEL) {
        clear_blur(&meter_window).ok();
    }
}

/// Copies the sync container data to the clipboard.
///
/// # Arguments
///
/// * `app` - A handle to the Tauri application instance.
/// * `state_manager` - The state manager.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result.
#[tauri::command]
#[specta::specta]
pub async fn copy_sync_container_data(
    app: tauri::AppHandle,
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<(), String> {
    let encounter = state_manager.get_encounter().await;
    let json = serde_json::to_string_pretty(&encounter.local_player)
        .map_err(|e| format!("Failed to serialize data: {}", e))?;
    app.clipboard()
        .write_text(json)
        .map_err(|e| format!("Failed to write to clipboard: {}", e))?;
    Ok(())
}

// #[tauri::command]
// #[specta::specta]
// pub fn get_header_info(state: tauri::State<'_, EncounterMutex>) -> Result<HeaderInfo, String> {
//     let encounter = state.lock().unwrap();

//     if encounter.total_dmg == 0 {
//         return Err("No damage found".to_string());
//     }

//     let time_elapsed_ms = encounter
//         .time_last_combat_packet_ms
//         .saturating_sub(encounter.time_fight_start_ms);
//     #[allow(clippy::cast_precision_loss)]
//     let time_elapsed_secs = time_elapsed_ms as f64 / 1000.0;

//     #[allow(clippy::cast_precision_loss)]
//     Ok(HeaderInfo {
//         total_dps: nan_is_zero(encounter.total_dmg as f64 / time_elapsed_secs),
//         total_dmg: encounter.total_dmg,
//         elapsed_ms: time_elapsed_ms,
//     })
// }

// #[tauri::command]
// #[specta::specta]
// pub fn hard_reset(state: tauri::State<'_, EncounterMutex>) {
//     let mut encounter = state.lock().unwrap();
//     encounter.clone_from(&Encounter::default());
//     request_restart();
//     info!("Hard Reset");
// }

/// Resets the encounter.
///
/// # Arguments
///
/// * `state_manager` - The state manager.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result.
#[tauri::command]
#[specta::specta]
pub async fn reset_encounter(
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<(), String> {
    let state_manager = state_manager.inner().clone();
    tauri::async_runtime::spawn(async move {
        // Use the centralized state event handler so that the EndEncounter DB task
        // is enqueued and all side-effects (emit events, clear subscriptions) are
        // handled consistently.
        state_manager
            .handle_event(StateEvent::ResetEncounter { is_manual: true })
            .await;
        info!("encounter reset via command");
    });
    Ok(())
}

/// Toggles pausing the encounter.
///
/// # Arguments
///
/// * `state_manager` - The state manager.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result.
#[tauri::command]
#[specta::specta]
pub async fn toggle_pause_encounter(
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<(), String> {
    let state_manager = state_manager.inner().clone();
    tauri::async_runtime::spawn(async move {
        // Read current paused state and delegate to centralized handler which
        // will update the state and emit events as appropriate.
        let is_paused = state_manager
            .with_state(|state| state.encounter.is_encounter_paused)
            .await;
        state_manager
            .handle_event(StateEvent::PauseEncounter(!is_paused))
            .await;
    });
    Ok(())
}

/// Resets player metrics for the live meter without ending the encounter.
/// This is used for segment transitions to clear UI data.
///
/// # Arguments
///
/// * `state_manager` - The state manager.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result.
#[tauri::command]
#[specta::specta]
pub async fn reset_player_metrics(
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<(), String> {
    use crate::live::commands_models::HeaderInfo;

    state_manager
        .with_state_mut(|state| {
            // Store the original fight start time before reset
            let original_fight_start_ms = state.encounter.time_fight_start_ms;

            // Reset combat state (player metrics)
            // Grab current active segment name (if any) so it can be included in the
            // emitted event payload for frontend to display a toast text notification.
            let active_segment_name = dungeon_log::snapshot(&state.dungeon_log).and_then(|log| {
                log.segments
                    .iter()
                    .rev()
                    .find(|s| s.ended_at_ms.is_none())
                    .and_then(|s| s.boss_name.clone())
            });
            state.encounter.reset_combat_state();
            state.skill_subscriptions.clear();

            // Restore the original fight start time to preserve total encounter duration
            state.encounter.time_fight_start_ms = original_fight_start_ms;

            // Emit reset event to clear frontend stores
            if state.event_manager.should_emit_events() {
                // Emit a player-metrics-only reset event for the current segment.
                // resets with full encounter resets (e.g., server change/Scene change).
                state
                    .event_manager
                    .emit_player_metrics_reset(active_segment_name);

                // Emit an encounter update with cleared player data but preserve encounter context
                let cleared_header = HeaderInfo {
                    total_dps: 0.0,
                    total_dmg: 0,
                    elapsed_ms: 0,
                    fight_start_timestamp_ms: state.encounter.time_fight_start_ms,
                    bosses: vec![],
                    scene_id: state.encounter.current_scene_id,
                    scene_name: state.encounter.current_scene_name.clone(),
                    current_segment_type: None,
                    current_segment_name: None,
                };
                state
                    .event_manager
                    .emit_encounter_update(cleared_header, state.encounter.is_encounter_paused);
            }
        })
        .await;

    info!("Player metrics reset for segment transition");
    Ok(())
}

/// Sets whether wipe detection is enabled.
///
/// # Arguments
///
/// * `enabled` - Whether to enable wipe detection.
/// * `state_manager` - The state manager.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result.
#[tauri::command]
#[specta::specta]
pub async fn set_wipe_detection_enabled(
    enabled: bool,
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<(), String> {
    state_manager
        .with_state_mut(|state| {
            state.attempt_config.enable_wipe_detection = enabled;
        })
        .await;
    info!("Wipe detection enabled: {}", enabled);
    Ok(())
}

/// Returns the current active buffs for all players in the encounter.
#[tauri::command]
#[specta::specta]
pub async fn get_live_buffs(
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<Vec<crate::live::commands_models::EntityBuffsDto>, String> {
    use crate::live::buff_names;
    use crate::live::commands_models::{BuffEventDto, BuffInfoDto, EntityBuffsDto};
    use crate::live::opcodes_models::AttrType;
    use blueprotobuf_lib::blueprotobuf::EEntityType;

    let state = state_manager.state.read().await;
    let encounter = &state.encounter;

    let mut result_map: std::collections::HashMap<i64, EntityBuffsDto> =
        std::collections::HashMap::new();

    for ((entity_id, buff_id), events) in &encounter.buff_events {
        // Check if entity is a player
        let entity_name = if let Some(entity) = encounter.entity_uid_to_entity.get(entity_id) {
            if entity.entity_type != EEntityType::EntChar {
                continue;
            }
            // Try to get name from attributes
            if let Some(attr) = entity.attributes.get(&AttrType::Name) {
                attr.as_string().unwrap_or("Unknown").to_string()
            } else {
                format!("Player {}", entity_id)
            }
        } else {
            // If entity not found, skip (likely not a player we track, or stale)
            continue;
        };

        let entry = result_map
            .entry(*entity_id)
            .or_insert_with(|| EntityBuffsDto {
                entity_uid: *entity_id,
                entity_name: entity_name.clone(),
                buffs: Vec::new(),
            });

        if let Some((buff_short, buff_long)) = buff_names::lookup_full(*buff_id) {
            let event_dtos: Vec<BuffEventDto> = events
            .iter()
            .map(|e| BuffEventDto {
                start_ms: e.start,
                end_ms: e.end,
                duration_ms: e.duration as i64,
                stack_count: e.stack_count,
            })
            .collect();
            let total_duration_ms: i64 = event_dtos.iter().map(|e| e.duration_ms).sum();
            if total_duration_ms > 0 {
                entry.buffs.push(BuffInfoDto {
                    buff_id: *buff_id,
                    buff_name: buff_short,
                    buff_name_long: Some(buff_long),
                    total_duration_ms,
                    events: event_dtos,
                });
            }
        }
    }

    Ok(result_map.into_values().collect())
}
