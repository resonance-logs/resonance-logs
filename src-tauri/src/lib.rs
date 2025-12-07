mod build_app;
mod live;
mod packets;

use crate::build_app::build_and_run;
use log::{info, warn};
use specta_typescript::{BigIntExportBehavior, Typescript};
#[cfg(windows)]
use std::process::{Command, Stdio};

use chrono_tz;
#[cfg(not(debug_assertions))]
use log::LevelFilter;
use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{LogicalPosition, LogicalSize, Manager, Position, Size, Window, WindowEvent};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
// NOTE: the updater extension trait is imported next to the helper that uses it
// and is cfg-gated to avoid unused-import warnings on builds that don't enable
// the updater plugin.
use tauri_specta::{Builder, collect_commands};
mod database;
mod tracking;
mod uploader; // stage 4 upload logic
use serde_json::json;

/// The label for the live window.
pub const WINDOW_LIVE_LABEL: &str = "live";
/// The label for the main window.
pub const WINDOW_MAIN_LABEL: &str = "main";

/// The main entry point for the application logic.
///
/// This function sets up and runs the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // std::panic::set_hook(Box::new(|info| {
    //     info!pub(crate)("App crashed! Info: {:?}", info);
    //     unload_and_remove_windivert();
    // }));

    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            live::commands::enable_blur,
            live::commands::disable_blur,
            live::commands::reset_encounter,
            live::commands::toggle_pause_encounter,
            live::commands::reset_player_metrics,
            live::commands::get_player_skills,
            live::commands::subscribe_player_skills,
            live::commands::unsubscribe_player_skills,
            live::commands::set_boss_only_dps,
            live::commands::set_dungeon_segments_enabled,
            live::commands::set_wipe_detection_enabled,
            live::commands::get_dungeon_log,
            live::commands::get_live_buffs,
            database::commands::get_recent_encounters,
            database::commands::get_unique_scene_names,
            database::commands::get_unique_boss_names,
            database::commands::get_player_names_filtered,
            database::commands::get_recent_encounters_filtered,
            database::commands::get_encounter_actor_stats,
            database::commands::get_encounter_by_id,
            database::commands::get_encounter_player_skills,
            database::commands::get_encounter_segments,
            database::commands::delete_encounter,
            database::commands::delete_encounters,
            database::commands::toggle_favorite_encounter,
            database::commands::get_recent_players_command,
            database::commands::get_player_name_command,
            database::commands::get_encounter_buffs,
            uploader::start_upload,
            uploader::cancel_upload_cmd,
            uploader::player_data_sync::sync_player_data,
            packet_settings_commands::save_packet_capture_settings,
            packets::npcap::get_network_devices,
            packets::npcap::check_npcap_status,
            debug_commands::open_log_dir,
        ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(
            Typescript::new().bigint(BigIntExportBehavior::Number),
            "../src/lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    let tauri_builder = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            info!("starting app v{}", app.package_info().version);
            stop_windivert();
            remove_windivert();

            // Check app updates
            // https://v2.tauri.app/plugin/updater/#checking-for-updates
            // Only run updater checks on Windows builds (automatic apply)
            #[cfg(windows)]
            {
                // Unload driver to avoid file handle conflicts during update
                unload_and_remove_windivert();

                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = crate::check_for_updates(handle).await {
                        warn!("Updater error: {}", e);
                    }
                });
            }

            // Track application update (anonymous telemetry)
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    crate::tracking::track_update(handle).await;
                });
            }

            let app_handle = app.handle().clone();

            // Setup logs
            if let Err(e) = setup_logs(&app_handle) {
                warn!("Failed to setup logs: {}", e);
            }

            // Install panic hook to create a crash dump file when the app panics.
            // This is installed after logs so we can use the configured logger.
            let hook_app_handle = app_handle.clone();
            // Take the default panic hook so we can call it after our handling.
            let default_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                // Try to persist a crash dump to the app log directory.
                let backtrace = std::backtrace::Backtrace::force_capture();
                let package_version = hook_app_handle.package_info().version.clone();
                let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                let file_name = format!("crash_dump_v{}_{timestamp}.log", package_version);
                let mut dump_content = String::new();
                dump_content.push_str(&format!("Panic occurred: {}\n", info));
                dump_content.push_str(&format!("Backtrace:\n{:?}\n", backtrace));
                dump_content.push_str(&format!("OS: {} {}\n", std::env::consts::OS, std::env::consts::ARCH));
                // Prefer to write to the app-specific log directory; fall back to a standard OS directory.
                let path_targets = [
                    // Try app-specific data dir first (by vendor settings via dirs crate)
                    dirs::data_local_dir().map(|d| d.join("resonance-logs").join("logs")),
                    // Fallback to current working directory
                    std::env::current_dir().ok().map(|d| d.join("logs")),
                ];
                let mut written = false;
                for target in path_targets.into_iter().flatten() {
                    if let Err(e) = std::fs::create_dir_all(&target) {
                        warn!("Failed to create dump dir {}: {}", target.display(), e);
                        continue;
                    }
                    let file_path = target.join(&file_name);
                    match std::fs::write(&file_path, &dump_content) {
                        Ok(_) => {
                            warn!("Wrote crash dump to {}", file_path.display());
                            written = true;
                            break;
                        }
                        Err(e) => warn!("Failed to write crash dump to {}: {}", file_path.display(), e),
                    }
                }
                if !written {
                    warn!("Failed to write crash dump to any known location; printing dump content to logs instead");
                    warn!("Crash dump:\n{}", dump_content);
                }
                // Attempt a clean up of resources (driver) before handing off to default handler.
                unload_and_remove_windivert();
                // Call the previously installed panic hook (prints to stderr etc)
                default_hook(info);
            }));

            // Initialize database and background writer
            if let Err(e) = crate::database::init_and_spawn_writer() {
                warn!("Failed to initialize database: {}", e);
            }

            // Setup tray icon
            setup_tray(&app_handle).expect("failed to setup tray");

            // Create and manage the state manager
            let state_manager = crate::live::state::AppStateManager::new(app_handle.clone());
            app.manage(state_manager.clone());

            // Auto-upload state (combat log uploader)
            let auto_upload_state = crate::uploader::AutoUploadState::default();
            app.manage(auto_upload_state.clone());

            // Player data sync state (separate from encounter uploads)
            let player_data_sync_state = crate::uploader::player_data_sync::PlayerDataSyncState::default();
            app.manage(player_data_sync_state.clone());

            crate::uploader::start_auto_upload_task(app_handle.clone(), auto_upload_state.clone());
            // Start player data sync background task (runs every 15 minutes)
            crate::uploader::player_data_sync::start_player_data_sync_task(app_handle.clone(), player_data_sync_state.clone());

            // Live Meter
            // https://v2.tauri.app/learn/splashscreen/#start-some-setup-tasks
            tauri::async_runtime::spawn(
                async move { live::live_main::start(app_handle.clone()).await },
            );
            Ok(())
        })
        .on_window_event(on_window_event_fn)
        .plugin(tauri_plugin_clipboard_manager::init()) // used to read/write to the clipboard
        .plugin(tauri_plugin_window_state::Builder::default().build()) // used to remember window size/position https://v2.tauri.app/plugin/window-state/
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {})) // used to enforce only 1 instance of the app https://v2.tauri.app/plugin/single-instance/
        .plugin(tauri_plugin_opener::init()) // used to open URLs in the default browser
        .plugin(tauri_plugin_svelte::init()); // used for settings file
    build_and_run(tauri_builder);
}

mod packet_settings_commands {
    use super::*;

    #[tauri::command]
    #[specta::specta]
    pub fn save_packet_capture_settings(
        method: String,
        npcap_device: String,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        let app_data_dirs = [
            app_handle.path().app_data_dir(),
            app_handle.path().app_local_data_dir(),
        ];
        let mut last_err = None;

        for dir in app_data_dirs.into_iter().flatten() {
            let target_dir = dir.join("stores");
            if let Err(e) = std::fs::create_dir_all(&target_dir) {
                last_err = Some(format!("create_dir_all {}: {}", target_dir.display(), e));
                continue;
            }
            let path = target_dir.join("packetCapture.json");
            let payload = json!({
                "method": method,
                "npcapDevice": npcap_device,
            });
            match std::fs::write(
                &path,
                serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?,
            ) {
                Ok(_) => {
                    info!("Saved packet capture config to {}", path.display());
                    return Ok(());
                }
                Err(e) => last_err = Some(format!("write {}: {}", path.display(), e)),
            }
        }

        Err(last_err.unwrap_or_else(|| "Failed to save packet capture config".to_string()))
    }
}

mod debug_commands {
    use super::*;

    #[tauri::command]
    #[specta::specta]
    pub fn open_log_dir(app_handle: tauri::AppHandle) -> Result<(), String> {
        let log_dir = app_handle
            .path()
            .app_log_dir()
            .map_err(|e| format!("Failed to get log dir: {}", e))?;

        if !log_dir.exists() {
            return Err("Log directory does not exist".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .arg(&log_dir)
                .spawn()
                .map_err(|e| format!("Failed to open log dir: {}", e))?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            // For other OSs, we can use 'open' (macOS) or 'xdg-open' (Linux)
            // But since this is a Windows-focused request, I'll essentially leave it as a no-op or specific to Windows for now based on user context.
            // But good to have a fallback or error.
            // Using `open` crate or tauri's `open` plugin would be better but let's stick to simple Command for now as requested.
            // Actually, tauri_plugin_opener is initialized in lib.rs, so we might utilize that if we want, but 'explorer' is specific.
            Command::new("xdg-open")
                .arg(&log_dir)
                .spawn()
                .map_err(|e| format!("Failed to open log dir: {}", e))?;
        }

        Ok(())
    }
}

/// Starts the WinDivert driver.
///
/// This function executes a shell command to create and start the WinDivert driver service.
fn start_windivert() {
    // Run the command silently (no console window) on Windows. On other platforms, just
    // redirect stdio to null so nothing is printed.
    let mut cmd = Command::new("sc");
    cmd.args([
        "create",
        "windivert",
        "type=",
        "kernel",
        "binPath=",
        "WinDivert64.sys",
        "start=",
        "demand",
    ]);
    let status = run_command_silently(&mut cmd);
    if status.is_ok_and(|status| status.success()) {
        info!("started driver");
    } else {
        warn!("could not execute command to stop driver");
    }
}

/// Stops the WinDivert driver.
///
/// This function executes a shell command to stop the WinDivert driver service.
fn stop_windivert() {
    let mut cmd = Command::new("sc");
    cmd.args(["stop", "windivert"]);
    let status = run_command_silently(&mut cmd);
    if status.is_ok_and(|status| status.success()) {
        info!("stopped driver");
    } else {
        warn!("could not execute command to stop driver");
    }
}

/// Removes the WinDivert driver.
///
/// This function executes a shell command to delete the WinDivert driver service.
fn remove_windivert() {
    let mut cmd = Command::new("sc");
    cmd.args(["delete", "windivert", "start=", "demand"]);
    let status = run_command_silently(&mut cmd);
    if status.is_ok_and(|status| status.success()) {
        info!("deleted driver");
    } else {
        warn!("could not execute command to delete driver");
    }
}

/// Helper to unload and remove the WinDivert driver.
///
/// On Windows this attempts to stop and delete the service. On other
/// platforms this is a no-op.
fn unload_and_remove_windivert() {
    #[cfg(windows)]
    {
        // Try to stop and remove the driver; these helpers already log
        // warnings on failure so we don't need to handle the results here.
        stop_windivert();
        remove_windivert();
    }
    #[cfg(not(windows))]
    {
        // no-op on non-windows platforms
    }
}

/// Helper to run a prepared Command with stdio redirected to null and (on Windows)
/// with the CREATE_NO_WINDOW flag so no console window appears.
fn run_command_silently(cmd: &mut Command) -> std::io::Result<std::process::ExitStatus> {
    #[cfg(windows)]
    {
        // CREATE_NO_WINDOW = 0x08000000
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .status()
    }

    #[cfg(not(windows))]
    {
        cmd.stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .status()
    }
}

// Updater helper: checks for updates and downloads+installs them automatically.
// This runs only on Windows builds (guarded where it is invoked).
#[cfg(windows)]
use tauri_plugin_updater::UpdaterExt;

#[cfg(windows)]
async fn check_for_updates(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    // If an update is available, download and install it, then restart the app.
    if let Some(update) = app.updater()?.check().await? {
        info!("Update available, starting download and install...");
        // Provide a simple logging progress callback. Use the provided
        // download_and_install helper from the updater plugin which applies the update.
        update
            .download_and_install::<_, _>(
                |chunk_length: usize, content_length: Option<u64>| {
                    info!(
                        "downloaded {} bytes (total: {:?})",
                        chunk_length, content_length
                    );
                },
                || {
                    info!("download finished");
                },
            )
            .await?;

        info!("Update installed successfully, restarting application...");
        app.restart();
    } else {
        info!("No update available");
    }
    Ok(())
}

/// Sets up the logging for the application.
///
/// This function configures the logging targets and settings.
///
/// # Arguments
///
/// * `app` - A handle to the Tauri application instance.
///
/// # Returns
///
/// * `tauri::Result<()>` - An empty result indicating success or failure.
fn setup_logs(app: &tauri::AppHandle) -> tauri::Result<()> {
    let app_version = &app.package_info().version;
    let pst_time = chrono::Utc::now()
        .with_timezone(&chrono_tz::America::Los_Angeles)
        .format("%m-%d-%Y %H_%M_%S")
        .to_string();
    let log_file_name = format!("log v{app_version} {pst_time} PST",);

    let mut tauri_log = tauri_plugin_log::Builder::new() // https://v2.tauri.app/plugin/logging/
        .clear_targets()
        .with_colors(ColoredLevelConfig::default())
        .targets([
            #[cfg(debug_assertions)]
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout)
                .filter(|metadata| metadata.level() <= log::LevelFilter::Trace),
            // LogDir target - in debug builds log everything; in release builds only warnings/errors
            {
                let target = tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                    file_name: Some(log_file_name),
                });
                #[cfg(debug_assertions)]
                let target = target.filter(|metadata| metadata.level() <= log::LevelFilter::Trace);
                #[cfg(not(debug_assertions))]
                let target = target.filter(|metadata| metadata.level() <= LevelFilter::Warn);
                target
            },
        ])
        .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(10)); // keep the last 10 logs
    #[cfg(not(debug_assertions))]
    {
        tauri_log = tauri_log.max_file_size(1_073_741_824 /* 1 gb */);
    }
    app.plugin(tauri_log.build())?;
    Ok(())
}

/// Sets up the system tray icon and menu.
///
/// This function creates the tray icon, defines its menu, and sets up event handlers.
///
/// # Arguments
///
/// * `app` - A handle to the Tauri application instance.
///
/// # Returns
///
/// * `tauri::Result<()>` - An empty result indicating success or failure.
fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    fn show_window_and_disable_clickthrough(window: &tauri::WebviewWindow) {
        if let Err(e) = window.show() {
            warn!("failed to show window {}: {}", window.label(), e);
        }
        if let Err(e) = window.unminimize() {
            warn!("failed to unminimize window {}: {}", window.label(), e);
        }
        if let Err(e) = window.set_focus() {
            warn!("failed to focus window {}: {}", window.label(), e);
        }
        // Always disable clickthrough when showing window from tray
        if window.label() == WINDOW_LIVE_LABEL {
            if let Err(e) = window.set_ignore_cursor_events(false) {
                warn!(
                    "failed to set ignore_cursor_events for {}: {}",
                    window.label(),
                    e
                );
            }
        }
    }

    let menu = MenuBuilder::new(app)
        .text("show-settings", "Show Settings")
        .separator()
        .text("show-live", "Show Live Meter")
        .text("reset", "Reset Window")
        .text("clickthrough", "Disable Clickthrough")
        .separator()
        .text("quit", "Quit")
        .build()?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|tray_app, event| match event.id.as_ref() {
            "show-settings" => {
                let tray_app_handle = tray_app.app_handle();
                let Some(main_meter_window) = tray_app_handle.get_webview_window(WINDOW_MAIN_LABEL)
                else {
                    return;
                };
                show_window_and_disable_clickthrough(&main_meter_window);
            }
            "show-live" => {
                let tray_app_handle = tray_app.app_handle();
                let Some(live_meter_window) = tray_app_handle.get_webview_window(WINDOW_LIVE_LABEL)
                else {
                    return;
                };
                show_window_and_disable_clickthrough(&live_meter_window);
            }
            "reset" => {
                let Some(live_meter_window) = tray_app.get_webview_window(WINDOW_LIVE_LABEL) else {
                    return;
                };
                if let Err(e) = live_meter_window.set_size(Size::Logical(LogicalSize {
                    width: 500.0,
                    height: 350.0,
                })) {
                    warn!("failed to resize live window: {}", e);
                }
                if let Err(e) = live_meter_window
                    .set_position(Position::Logical(LogicalPosition { x: 100.0, y: 100.0 }))
                {
                    warn!("failed to set position for live window: {}", e);
                }
                if let Err(e) = live_meter_window.show() {
                    warn!("failed to show live window: {}", e);
                }
                if let Err(e) = live_meter_window.unminimize() {
                    warn!("failed to unminimize live window: {}", e);
                }
                if let Err(e) = live_meter_window.set_focus() {
                    warn!("failed to focus live window: {}", e);
                }
                if let Err(e) = live_meter_window.set_ignore_cursor_events(false) {
                    warn!("failed to set ignore_cursor_events for live window: {}", e);
                }
            }
            "clickthrough" => {
                let Some(live_meter_window) = tray_app.get_webview_window(WINDOW_LIVE_LABEL) else {
                    return;
                };
                if let Err(e) = live_meter_window.set_ignore_cursor_events(false) {
                    warn!("failed to set ignore_cursor_events for live window: {}", e);
                }
            }
            "quit" => {
                stop_windivert();
                tray_app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // Show and focus the live meter window when the tray is clicked
                // Also disable clickthrough mode to make it interactable
                let app = tray.app_handle();
                let Some(live_meter_window) = app.get_webview_window(WINDOW_LIVE_LABEL) else {
                    return;
                };
                show_window_and_disable_clickthrough(&live_meter_window);
            }
        })
        .build(app)?;
    Ok(())
}

/// Handles window events.
///
/// This function is called whenever a window event occurs.
///
/// # Arguments
///
/// * `window` - The window that received the event.
/// * `event` - The event that occurred.
fn on_window_event_fn(window: &Window, event: &WindowEvent) {
    match event {
        // when you click the X button to close a window
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close(); // don't close it, just hide it
            if window.label() == WINDOW_MAIN_LABEL {
                if let Err(e) = window.hide() {
                    warn!("failed to hide main window: {}", e);
                }
            }
        }
        WindowEvent::Focused(focused) if !focused => {
            if let Err(e) = window.app_handle().save_window_state(StateFlags::all()) {
                warn!("failed to save window state for {}: {}", window.label(), e);
            }
        }
        _ => {}
    }
}
