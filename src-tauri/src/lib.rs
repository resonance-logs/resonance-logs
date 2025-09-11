mod app;
mod live;
mod packets;

use log::{info, Level};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// # Panics
///
/// Will panic if todo:
pub fn run() {
    // init logger - we want 1 logger trace that goes to file, 1 logger info that goes to console todo: put it somewhere else
    tauri::Builder::default()
        .setup(|app| {
            // let app_handle = app.handle().clone();
            // app::init_app(app_handle);

            info!("starting app v{}", app.package_info().version);
            // TODO: Setup auto updater

            // Live Meter
            // https://v2.tauri.app/learn/splashscreen/#start-some-setup-tasks
            // let app_handle = app.app_handle().clone(); // todo:
            tauri::async_runtime::spawn(async move { live::live_main::start().await });
            Ok(())
        })
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {})) // https://v2.tauri.app/plugin/single-instance/
        // todo: remove after maturity of other logging
        .plugin(
            tauri_plugin_log::Builder::new() // https://v2.tauri.app/plugin/logging/
                .clear_targets()
                .with_colors(ColoredLevelConfig::default())
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout)
                        .filter(|metadata| metadata.level() <= log::LevelFilter::Info),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some(format!(
                            "{:?}",
                            chrono::Utc::now()
                                .timestamp_nanos_opt()
                                .expect("time out of bounds")
                        )),
                    }),
                ])
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
