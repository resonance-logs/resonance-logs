use crate::WINDOW_LIVE_LABEL;
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
            match skill_type.as_str() {
                "dps" => {
                    event_manager::generate_skills_window_dps(&state.encounter, uid, boss_only)
                        .ok_or_else(|| format!("No DPS skills found for player {}", uid))
                }
                "heal" => event_manager::generate_skills_window_heal(&state.encounter, uid)
                    .ok_or_else(|| format!("No heal skills found for player {}", uid)),
                "tanked" => event_manager::generate_skills_window_tanked(&state.encounter, uid)
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
            match skill_type.as_str() {
                "dps" => {
                    event_manager::generate_skills_window_dps(&state.encounter, uid, boss_only)
                        .ok_or_else(|| format!("No DPS skills found for player {}", uid))
                }
                "heal" => event_manager::generate_skills_window_heal(&state.encounter, uid)
                    .ok_or_else(|| format!("No heal skills found for player {}", uid)),
                "tanked" => event_manager::generate_skills_window_tanked(&state.encounter, uid)
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
        state_manager.handle_event(StateEvent::ResetEncounter).await;
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

/// Response for get_encounter_phases command.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterPhase {
    pub id: i32,
    pub encounter_id: i32,
    pub phase_type: String,
    pub start_time_ms: i64,
    pub end_time_ms: Option<i64>,
    pub outcome: String,
}

/// Gets all phases for a specific encounter.
///
/// # Arguments
///
/// * `encounter_id` - The ID of the encounter.
///
/// # Returns
///
/// * `Result<Vec<EncounterPhase>, String>` - A list of encounter phases.
#[tauri::command]
#[specta::specta]
pub fn get_encounter_phases(encounter_id: i32) -> Result<Vec<EncounterPhase>, String> {
    use crate::database::schema::encounter_phases::dsl as ep;
    use crate::database::{default_db_path, ensure_parent_dir};
    use diesel::prelude::*;

    let path = default_db_path();
    ensure_parent_dir(&path).map_err(|e| e.to_string())?;
    let mut conn = diesel::sqlite::SqliteConnection::establish(&path.to_string_lossy())
        .map_err(|e| e.to_string())?;

    let phases: Vec<(i32, i32, String, i64, Option<i64>, String)> = ep::encounter_phases
        .filter(ep::encounter_id.eq(encounter_id))
        .order_by(ep::start_time_ms.asc())
        .select((
            ep::id,
            ep::encounter_id,
            ep::phase_type,
            ep::start_time_ms,
            ep::end_time_ms,
            ep::outcome,
        ))
        .load::<(i32, i32, String, i64, Option<i64>, String)>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(phases
        .into_iter()
        .map(|(id, enc_id, phase_type, start_time_ms, end_time_ms, outcome)| EncounterPhase {
            id,
            encounter_id: enc_id,
            phase_type,
            start_time_ms,
            end_time_ms,
            outcome,
        })
        .collect())
}
