use crate::live::event_manager::EventManagerMutex;
use crate::live::opcodes_models::{Encounter, EncounterMutex};
use crate::WINDOW_LIVE_LABEL;
use log::info;
use tauri::Manager;
use window_vibrancy::{apply_blur, clear_blur};

#[tauri::command]
#[specta::specta]
pub fn disable_blur(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(WINDOW_LIVE_LABEL) {
        clear_blur(&meter_window).ok();
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
