mod live;
mod packets;

use blueprotobuf_lib::blueprotobuf::EEntityType;
use log::{info};
use tauri::{Manager};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use crate::live::models::{Encounter, EncounterInner};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// # Panics
///
/// Will panic if todo:
pub fn run() {
    let time_now = Some(format!(
        "{:?}",
        chrono::Utc::now()
            .timestamp_nanos_opt()
            .expect("time out of bounds")
    ));

    tauri::Builder::default()
        .setup(|app| {
            info!("starting app v{}", app.package_info().version);
            app.manage(Encounter::default()); // todo: maybe use https://github.com/ferreira-tb/tauri-store

            // TODO: Setup auto updater

            // Live Meter
            // https://v2.tauri.app/learn/splashscreen/#start-some-setup-tasks
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move { live::live_main::start(app_handle.clone()).await });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_encounter])
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {})) // https://v2.tauri.app/plugin/single-instance/
        .plugin(
            tauri_plugin_log::Builder::new() // https://v2.tauri.app/plugin/logging/
                .clear_targets()
                .with_colors(ColoredLevelConfig::default())
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout)
                        .filter(|metadata| metadata.level() <= log::LevelFilter::Info),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir { file_name: time_now }),
                ])
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_encounter(state: tauri::State<'_, Encounter>) -> EncounterInner {
    let state = state.lock().unwrap();
    let mut state_clone = state.clone();
    state_clone.entities.retain(|_uuid, entity| {
        let is_player = entity.entity_type == EEntityType::EntChar;
        let did_damage = entity.damage_stats.damage_dealt > 0;
        is_player && did_damage
    });

    state_clone
}
