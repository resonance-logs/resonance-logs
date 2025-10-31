# Architecture Overview

## High-Level System
- **Desktop shell** – A Tauri 2 runner initializes plugins, tray controls, and window management before delegating to the Svelte frontend. The entry point exports TypeScript bindings during debug builds and seeds background tasks for packet capture and logging.
- **Rust backend** – Handles packet ingestion, live encounter state, and persistence. Network traffic flows through WinDivert into the packet pipeline, which produces typed encounter updates and schedules database writes.
- **Svelte frontend** – Provides two WebView windows: the "main" settings/history interface and the "live" overlay. Both consume Tauri commands and events exposed by the Rust layer.

```
 ┌────────────┐       ┌──────────────────────┐       ┌───────────────────────────┐
 │  Game Net  │──────▶│ Packet capture (Rust)│──────▶│ Encounter state manager    │
 └────────────┘       └──────────────────────┘       └──────────────┬────────────┘
                                                                    │
                    ┌────────────────────────┐                      │
                    │ SQLite via Diesel ORM  │◀─────────────────────┘
                    └────────────────────────┘
                               │
                               ▼
                     ┌────────────────────┐
                     │ Svelte frontends   │
                     │ (main & live UIs)  │
                     └────────────────────┘
```

## Desktop Shell (Tauri)
- `run()` registers plugins (OS info, global shortcuts, window state, clipboard, single-instance, tauri-plugin-svelte for RuneStore) and configures tray menu behaviour.
- WinDivert lifecycle helpers ensure the driver is stopped/deleted before the app starts and when the user quits.
- Window events are intercepted so closing the main window hides the UI but leaves the process alive. The live window automatically stores size/position via the window-state plugin.

## Packet Processing Flow
1. `live_main::start()` launches asynchronous packet capture over WinDivert.
2. Each packet opcode is decoded into protobuf structs via the `blueprotobuf-lib` (local workspace crate) and converted into `StateEvent` variants.
3. `AppStateManager::handle_event()` mutates the encounter model, triggers persistence tasks, and maintains pause/reset semantics.
4. `update_and_emit_events()` generates derived DPS/heal/tanked windows plus skill breakdowns before emitting structured events to the front-end WebViews.

### Blueprotobuf Library
- A local crate in `src-tauri/src/blueprotobuf-lib/` that provides protobuf message definitions for Blue Protocol: Star Resonance network packets.
- Auto-generated via prost from `.proto` definitions with serde support for serialization.

## Event Emission & Commands
- `EventManager` emits `encounter-update`, `players-update`, `skills-update`, and pause/reset notifications.
- Tauri commands (`resetEncounter`, `togglePauseEncounter`, `setBossOnlyDps`, history queries, etc.) are exposed through `bindings.ts` for typed consumption by the Svelte app.

## Persistence Layer
- SQLite (via Diesel) stores encounters, entities, skill aggregates, and boss snapshots.
- `init_and_spawn_writer()` applies migrations, configures WAL mode, and spawns a batching writer thread to handle database mutations without blocking the UI thread.

## Frontend Windows
- `main` window wraps the sidebar layout, header breadcrumbs, and history/settings routes. It registers global shortcuts via `$effect.pre()` hook for managing the live overlay.
- `live` window focuses on real-time player tables. It redirects to the DPS route on mount and responds to backend events emitted through the event bus.
- Both windows use RuneStore (from `@tauri-store/svelte`) for persistent settings that survive app restarts. Live meter data uses RuneStore wrappers configured for transient state.

## Generated Bindings
- Specta exports backend commands into `src/lib/bindings.ts`, giving the frontend a zero-boilerplate API surface for invoking Rust commands and subscribing to updates.
