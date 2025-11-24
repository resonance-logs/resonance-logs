mod build_app;
mod live;
mod module_extractor;
mod packets;

use crate::build_app::build_and_run;
use log::{info, warn};
use specta_typescript::{BigIntExportBehavior, Typescript};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

use chrono_tz;
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
mod uploader; // stage 4 upload logic

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
            live::commands::get_dungeon_log,
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
            database::commands::get_recent_players_command,
            database::commands::get_player_name_command,
            uploader::start_upload,
            uploader::cancel_upload_cmd,
            module_extractor::commands::set_module_sync_config,
            module_extractor::commands::get_module_sync_status,
            module_extractor::commands::trigger_module_sync,
            module_extractor::commands::retry_failed_uploads,
            module_extractor::commands::get_unknown_attributes,
            module_extractor::commands::clear_unknown_attributes,
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

            let app_handle = app.handle().clone();

            // Setup logs
            setup_logs(&app_handle);

            // Initialize database and background writer
            if let Err(e) = crate::database::init_and_spawn_writer() {
                warn!("Failed to initialize database: {}", e);
            }

            // Setup tray icon
            setup_tray(&app_handle).expect("failed to setup tray");

            // Create and manage the state manager
            let state_manager = crate::live::state::AppStateManager::new(app_handle.clone());
            app.manage(state_manager.clone());

            // Create and manage the module sync state
            let module_sync_state = crate::module_extractor::commands::ModuleSyncState::default();
            app.manage(module_sync_state.clone());

            // Auto-upload state (combat log uploader)
            let auto_upload_state = crate::uploader::AutoUploadState::default();
            app.manage(auto_upload_state.clone());

            // Start auto-sync timer
            tauri::async_runtime::spawn(crate::module_extractor::commands::start_auto_sync_timer(
                module_sync_state,
            ));
            crate::uploader::start_auto_upload_task(app_handle.clone(), auto_upload_state);

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
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                file_name: Some(log_file_name),
            }),
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
        window.show().unwrap();
        window.unminimize().unwrap();
        window.set_focus().unwrap();
        // Always disable clickthrough when showing window from tray
        if window.label() == WINDOW_LIVE_LABEL {
            window.set_ignore_cursor_events(false).unwrap();
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
                live_meter_window
                    .set_size(Size::Logical(LogicalSize {
                        width: 500.0,
                        height: 350.0,
                    }))
                    .unwrap();
                live_meter_window
                    .set_position(Position::Logical(LogicalPosition { x: 100.0, y: 100.0 }))
                    .unwrap();
                live_meter_window.show().unwrap();
                live_meter_window.unminimize().unwrap();
                live_meter_window.set_focus().unwrap();
                live_meter_window.set_ignore_cursor_events(false).unwrap();
            }
            "clickthrough" => {
                let Some(live_meter_window) = tray_app.get_webview_window(WINDOW_LIVE_LABEL) else {
                    return;
                };
                live_meter_window.set_ignore_cursor_events(false).unwrap();
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
                window.hide().unwrap();
            }
        }
        WindowEvent::Focused(focused) if !focused => {
            window
                .app_handle()
                .save_window_state(StateFlags::all())
                .unwrap();
        }
        _ => {}
    }
}
