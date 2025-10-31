# Backend Developer Guide

## Overview
The backend is a Rust crate embedded inside the Tauri runner. It manages packet capture, encounter state, data persistence, and the command/event surface exposed to the Svelte frontend. Entry point: `src-tauri/src/lib.rs`. @src-tauri/src/lib.rs#29-311

## Key Crates & Dependencies
- **tauri**: Window management, IPC, menu, tray support.
- **tokio**: Async runtime used for packet loop and background tasks.
- **diesel** + **diesel_migrations**: SQLite ORM and migration handling.
- **windivert**: Kernel driver bindings for capturing network packets.
- **specta** + **tauri-specta**: Generates TypeScript bindings for frontend consumption.
- **window-vibrancy**, **tauri-plugin-global-shortcut**, **tauri-plugin-window-state**: Platform integrations for UX features.
- **tauri-plugin-svelte**: Provides RuneStore for persistent settings shared between frontend and backend.
- **tauri-plugin-log**: Structured logging with rotation strategy.
- **chrono** + **chrono-tz**: Timestamp handling and timezone conversions.
- **parking_lot**: High-performance RwLock for state management.
- **uuid**: Unique identifier generation for encounters and entities.
- **ts-rs**, **prost**, **zstd**, **byteorder**: Serialization and compression utilities.

## Runner Lifecycle (`run()`)
1. Build Specta command registry and export TypeScript bindings (debug only).
2. Register Tauri plugins (OS info, shortcuts, clipboard, window state, single-instance, tauri-plugin-svelte for settings persistence).
3. Configure tray menu and WinDivert lifecycle helpers.
4. Initialize logging via `tauri-plugin-log` to rotating files in the app data directory.
5. Initialize database connection and spawn background writer thread.
6. Spawn `live_main::start()` as an async task to begin packet capture.
7. Manage window events to hide instead of closing. @src-tauri/src/lib.rs#29-311

## Packet Capture Pipeline
- `packets::packet_capture::start_capture()` opens WinDivert and returns an async channel of `(Pkt, Vec<u8>)` tuples.
- `live_main::start()` consumes that channel, decoding protobuf payloads (via `blueprotobuf`) into typed structs per opcode. @src-tauri/src/live/live_main.rs#1-104
- Each decoded packet is converted to a `StateEvent` and handed to `AppStateManager::handle_event()`.

### Opcode Handling
- `StateEvent::SyncNearEntities`, `SyncContainerData`, `SyncContainerDirtyData`, etc., translate to the encounter model through functions in `opcodes_process.rs`.
- `StateEvent::ServerChange` resets encounter context and flushes state to disk.
- Pausing/resuming is controlled by `StateEvent::PauseEncounter(bool)` which toggles updates.

## State Management
- `AppStateManager` wraps `AppState` inside an `RwLock` and provides helper APIs:
  - `handle_event`: mutates encounter, enqueues DB work, and triggers event emission. @src-tauri/src/live/state.rs#68-214
  - `update_and_emit_events`: generates derived data windows (players, skills) and emits Tauri events with throttling handled upstream. @src-tauri/src/live/state.rs#278-398
- `Encounter` (in `opcodes_models.rs`) tracks per-entity damage/heal/tank stats, boss info, timestamps, and pause flags.
- Skill subscriptions: a `HashSet<(uid, skill_type)>` ensures only opted-in players receive skill breakdown events.

## Event Emission
- `EventManager` encapsulates all Tauri emit logic:
  - `encounter-update` → header stats + pause state.
  - `players-update` → DPS/Heal/Tanked tables.
  - `skills-update` → per-player skill breakdown when subscribed.
  - `reset-encounter`, `pause-encounter` → control events for the live window. @src-tauri/src/live/event_manager.rs#17-125

## Persistence Layer
- `database::init_and_spawn_writer()` runs at startup to set up the connection pool (SQLite, WAL mode) and spawn a writer thread that batches tasks (max 100 events or 50ms). @src-tauri/src/database/mod.rs#68-195
- `DbTask` enum covers encounter lifecycle (begin/end), entity upserts, damage/heal inserts. @src-tauri/src/database/mod.rs#207-251
- Each task maps to Diesel insert/update queries defined in `database::models`. Aggregation steps (skill stats, boss summaries) are triggered when an encounter ends.
- Schema is mirrored in `database::schema.rs`, keep migrations in sync when adding columns. @src-tauri/src/database/schema.rs#1-161

## Tauri Commands
- Commands live in `live/commands.rs` and `database/commands.rs`. Annotate with `#[tauri::command]` and `#[specta::specta]` to expose to the frontend.
- Common commands:
  - `reset_encounter`, `toggle_pause_encounter`, `set_boss_only_dps` (runtime control).
  - `get_recent_encounters_filtered`, `get_encounter_actor_stats` (history API).
  - Subscription helpers: `subscribe_player_skills`, `unsubscribe_player_skills`, `get_player_skills`. @src-tauri/src/live/commands.rs#28-207

## Adding New Features
1. Extend the encounter model (`opcodes_models.rs`) if packet data needs new fields.
2. Process incoming packets in `opcodes_process.rs` to populate the new data.
3. Update aggregation logic in `event_manager.rs` and/or database writer to compute derived metrics.
4. Expose data to the frontend by:
   - Adding new fields to Specta DTOs.
   - Adding commands or events.
   - Regenerating `bindings.ts` (`npm run tauri dev` in debug does this automatically).
5. Mirror any persisted changes in Diesel migrations and `schema.rs`.

## Debugging & Logging
- Logs are stored in the Tauri log directory with filenames keyed by version + PST timestamp. @src-tauri/src/lib.rs#174-201
- Uses `tauri-plugin-log` with `RotationStrategy::KeepSome(10)` to maintain the last 10 log files.
- Log file size is capped at 1GB in release builds.
- Use `info!`, `warn!`, and `error!` macros from the `log` crate for structured logging.
- On Windows, ensure WinDivert driver has appropriate permissions; `stop_windivert()` and `remove_windivert()` manage service lifecycle when restarting. @src-tauri/src/lib.rs#113-151

## Testing Tips
- Run `npm run tauri dev` to start the full stack with hot reload.
- Use recorded packet data (where available) to feed the packet capture channel for repeatable tests.
- Consider enabling verbose logging temporarily via environment variables or conditional compilation for diagnosing packet parsing issues.
