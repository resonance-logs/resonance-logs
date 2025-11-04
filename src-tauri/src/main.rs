// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// The entry point of the application.
///
/// This function initializes the application and starts the main event loop.
fn main() {
    resonance_logs_lib::run()
}
