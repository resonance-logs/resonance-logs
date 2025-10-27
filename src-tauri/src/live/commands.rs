use crate::live::state::{AppStateManager, StateEvent};
use crate::WINDOW_LIVE_LABEL;
use log::info;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use window_vibrancy::{apply_blur, clear_blur};
// request_restart is not needed in this module at present
use crate::live::event_manager; // for generate_skills_window_*

fn prettify_name(player_uid: i64, local_player_uid: i64, player_name: &String) -> String {
    if player_uid == local_player_uid && player_name.is_empty() {
        String::from("You")
    } else if player_uid == local_player_uid && !player_name.is_empty() {
        format!("{player_name} (You)")
    } else {
        player_name.clone()
    }
}

fn nan_is_zero(value: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        0.0
    } else {
        value
    }
}

#[tauri::command]
#[specta::specta]
pub async fn subscribe_player_skills(
    uid: i64,
    skill_type: String,
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<crate::live::commands_models::SkillsWindow, String> {
    // Register subscription
    state_manager.update_skill_subscriptions(|subs| {
        subs.insert((uid, skill_type.clone()));
    }).await;

    // Compute and return initial window directly from Encounter
    state_manager.with_state(|state| {
        let boss_only = state.boss_only_dps;
        match skill_type.as_str() {
            "dps" => event_manager::generate_skills_window_dps(&state.encounter, uid, boss_only)
                .ok_or_else(|| format!("No DPS skills found for player {}", uid)),
            "heal" => event_manager::generate_skills_window_heal(&state.encounter, uid)
                .ok_or_else(|| format!("No heal skills found for player {}", uid)),
            "tanked" => event_manager::generate_skills_window_tanked(&state.encounter, uid)
                .ok_or_else(|| format!("No tanked skills found for player {}", uid)),
            _ => Err(format!("Invalid skill type: {}", skill_type))
        }
    }).await
}

#[tauri::command]
#[specta::specta]
pub async fn unsubscribe_player_skills(
    uid: i64,
    skill_type: String,
    state_manager: tauri::State<'_, AppStateManager>,
)-> Result<(), String> {
    state_manager.update_skill_subscriptions(|subs| {
        subs.remove(&(uid, skill_type));
    }).await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_player_skills(
    uid: i64,
    skill_type: String,
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<crate::live::commands_models::SkillsWindow, String> {
    state_manager.with_state(|state| {
        let boss_only = state.boss_only_dps;
        match skill_type.as_str() {
            "dps" => event_manager::generate_skills_window_dps(&state.encounter, uid, boss_only)
                .ok_or_else(|| format!("No DPS skills found for player {}", uid)),
            "heal" => event_manager::generate_skills_window_heal(&state.encounter, uid)
                .ok_or_else(|| format!("No heal skills found for player {}", uid)),
            "tanked" => event_manager::generate_skills_window_tanked(&state.encounter, uid)
                .ok_or_else(|| format!("No tanked skills found for player {}", uid)),
            _ => Err(format!("Invalid skill type: {}", skill_type))
        }
    }).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_boss_only_dps(
    enabled: bool,
    state_manager: tauri::State<'_, AppStateManager>,
) -> Result<(), String> {
    state_manager.with_state_mut(|state| {
        state.boss_only_dps = enabled;
    }).await;
    // Recompute and emit updates immediately
    state_manager.update_and_emit_events().await;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn enable_blur(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(WINDOW_LIVE_LABEL) {
        apply_blur(&meter_window, Some((10, 10, 10, 50))).ok();
    }
}

#[tauri::command]
#[specta::specta]
pub fn disable_blur(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(WINDOW_LIVE_LABEL) {
        clear_blur(&meter_window).ok();
    }
}

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

#[tauri::command]
#[specta::specta]
pub async fn reset_encounter(
    state_manager: tauri::State<'_, AppStateManager>,
)-> Result<(), String> {
    // Use the centralized state event handler so that the EndEncounter DB task
    // is enqueued and all side-effects (emit events, clear subscriptions) are
    // handled consistently.
    state_manager.handle_event(StateEvent::ResetEncounter).await;
    info!("encounter reset via command");
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_pause_encounter(
    state_manager: tauri::State<'_, AppStateManager>,
)-> Result<(), String> {
    // Read current paused state and delegate to centralized handler which
    // will update the state and emit events as appropriate
    let is_paused = state_manager.with_state(|state| state.encounter.is_encounter_paused).await;
    state_manager.handle_event(StateEvent::PauseEncounter(!is_paused)).await;
    Ok(())
}
