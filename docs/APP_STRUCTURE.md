# Application Structure

This document outlines the overall structure of the backend application, explaining the roles of the key files and modules.

## Entry Point: `main.rs`

The application entry point is `src-tauri/src/main.rs`. Its primary responsibility is to call the `run()` function from the `resonance_logs_lib` library. This separation keeps the main entry point clean and delegates the application setup and logic to the library.

```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// The entry point of the application.
///
/// This function initializes the application and starts the main event loop.
fn main() {
    resonance_logs_lib::run()
}
```

## Core Logic: `lib.rs`

The core logic of the application resides in `src-tauri/src/lib.rs`. This file is responsible for:

- **Tauri Application Setup:** It uses `tauri::Builder` to construct the application, configuring plugins, event handlers, and the main application loop.
- **Command Registration:** It registers all the commands that can be invoked from the frontend using `collect_commands!`. These commands are defined in the `live` and `database` modules.
- **Plugin Initialization:** It initializes several Tauri plugins, such as `tauri-plugin-os`, `tauri-plugin-window-state`, `tauri-plugin-single-instance`, and `tauri-plugin-svelte`.
- **Window Management:** It handles window events, such as the close request, to hide the window instead of closing it.
- **System Tray:** It sets up the system tray icon and menu, providing options to show the application windows, reset the live meter, and quit the application.
- **Logging:** It configures the logging for the application using `tauri-plugin-log`.
- **Database Initialization:** It initializes the database and spawns a background writer thread.
- **Live Meter Initialization:** It spawns an async task to start the live meter, which is responsible for real-time packet capture and processing.
- **WinDivert Management:** It includes functions to start, stop, and remove the `WinDivert` driver, which is used for packet sniffing on Windows.

## Modules

The application is divided into several modules, each with a specific responsibility:

### `live`

This module is responsible for real-time event processing. It receives data from the `packets` module, processes it, and emits events to the frontend. It also manages the application state.

### `database`

This module handles all database interactions. It uses `diesel` and `sqlite` to store and retrieve data. It includes the database schema, models, and a set of commands for the frontend to interact with the database.

### `packets`

This module is responsible for capturing and processing network packets. It uses `WinDivert` to capture packets and a custom reassembler to reconstruct TCP streams. It then parses the packets to extract game data.

### `blueprotobuf-lib`

This module contains the protobuf definitions for the game's data protocol. It uses `prost` to generate Rust code from the `.proto` files, which is then used to deserialize the captured packet data.
