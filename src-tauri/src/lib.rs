mod live;
mod packets;
mod app;

use std::fs::File;
use std::path::PathBuf;
use flexi_logger::FlexiLoggerError;
use tauri::Manager;
use tracing::{info};
use tracing_subscriber::{filter::{LevelFilter}, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// # Panics
///
/// Will panic if todo:
pub fn run() {
    // init logger - we want 1 logger trace that goes to file, 1 logger info that goes to console todo: put it somewhere else
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            app::init_app(app_handle);

            info!("starting app v{}", app.package_info().version);
            // TODO: Setup auto updater

            // Live Meter
            // https://v2.tauri.app/learn/splashscreen/#start-some-setup-tasks
            // let app_handle = app.app_handle().clone(); // todo:
            tauri::async_runtime::spawn(async move {live::live_main::start()});
            Ok(())
        })
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {})) // https://v2.tauri.app/plugin/single-instance/
        // todo: remove after maturity of other logging
        // .plugin(
        //     // https://v2.tauri.app/plugin/logging/
        //     tauri_plugin_log::Builder::new()
        //         .clear_targets()
        //         .with_colors(ColoredLevelConfig::default())
        //         .target(tauri_plugin_log::Target::new(
        //             tauri_plugin_log::TargetKind::Stdout,
        //         ))
        //         .level(log::LevelFilter::Info)
        //         .target(tauri_plugin_log::Target::new(
        //             tauri_plugin_log::TargetKind::LogDir {
        //                 file_name: Some(format!(
        //                     "{:?}",
        //                     chrono::Utc::now()
        //                         .timestamp_nanos_opt()
        //                         .expect("time out of bounds")
        //                 )),
        //             },
        //         ))
        //         .level_for("tauri-plugin-log::file", log::LevelFilter::Trace)
        //         .build(),
        // )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


