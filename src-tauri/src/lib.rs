mod live;
mod packets;

use crate::live::opcodes_models::EncounterMutex;
use tauri::Manager;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;

use specta_typescript::{BigIntExportBehavior, Typescript};
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let time_now = Some(format!(
        "{:?}",
        chrono::Utc::now()
            .timestamp_nanos_opt()
            .expect("time out of bounds")
    ));

    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![live::commands::get_damage_row,]);


    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::new().bigint(BigIntExportBehavior::BigInt), "../src/lib/bindings.ts")
        .expect("Failed to export typescript bindings");


    tauri::Builder::default()
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            // info!("starting app v{}", app.package_info().version);
            app.manage(EncounterMutex::default()); // todo: maybe use https://github.com/ferreira-tb/tauri-store

            // TODO: Setup auto updater

            // Live Meter
            // https://v2.tauri.app/learn/splashscreen/#start-some-setup-tasks
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move { live::live_main::start(app_handle.clone()).await });
            Ok(())
        })
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
