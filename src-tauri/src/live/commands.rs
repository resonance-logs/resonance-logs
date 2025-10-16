use crate::live::event_manager::EventManagerMutex;
use crate::live::opcodes_models::{Encounter, EncounterMutex};
use std::sync::MutexGuard;
use crate::WINDOW_LIVE_LABEL;
use log::info;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use window_vibrancy::{apply_blur, clear_blur};
use crate::packets::packet_capture::request_restart;

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
pub fn copy_sync_container_data(app: tauri::AppHandle) {
    let state = app.state::<EncounterMutex>();
    let encounter = state.lock().unwrap();
    let json = serde_json::to_string_pretty(&encounter.local_player).unwrap();
    app.clipboard().write_text(json).unwrap();
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
pub fn reset_encounter(
    state: tauri::State<'_, EncounterMutex>,
    event_manager: tauri::State<'_, EventManagerMutex>,
) {
    let mut encounter = state.lock().unwrap();
    encounter.clone_from(&Encounter::default());
    info!("encounter reset");

    // Emit encounter reset event
    let event_manager = event_manager.lock().unwrap();
    if event_manager.should_emit_events() {
        event_manager.emit_encounter_reset();
    }
}

#[tauri::command]
#[specta::specta]
pub fn toggle_pause_encounter(
    state: tauri::State<'_, EncounterMutex>,
    event_manager: tauri::State<'_, EventManagerMutex>,
) {
    let mut encounter = state.lock().unwrap();
    let _was_paused = encounter.is_encounter_paused;
    encounter.is_encounter_paused = !encounter.is_encounter_paused;

    // Emit pause/resume event
    let event_manager = event_manager.lock().unwrap();
    if event_manager.should_emit_events() {
        event_manager.emit_encounter_pause(encounter.is_encounter_paused);
    }
}
