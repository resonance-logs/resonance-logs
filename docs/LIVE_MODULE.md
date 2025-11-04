# Live Module

The `live` module is the core of the real-time functionality of the Resonance Logs application. It is responsible for processing the captured game data, managing the application's state, and emitting events to the frontend for display in the live meter.

## Entry Point: `live_main.rs`

The entry point for the live module is the `start` function in `src-tauri/src/live/live_main.rs`. This asynchronous function is spawned as a background task during the application's setup in `lib.rs`.

The `start` function performs the following key tasks:

1.  **State Manager Initialization:** It retrieves the `AppStateManager` instance from the Tauri application state.
2.  **Packet Capture:** It starts the packet capture process by calling `packets::packet_capture::start_capture()`, which returns a channel receiver (`rx`).
3.  **Packet Processing Loop:** It enters a loop, continuously receiving packets from the `rx` channel.
4.  **Packet Decoding:** For each received packet, it decodes the data using the protobuf definitions from `blueprotobuf-lib`.
5.  **Event Handling:** It converts the decoded packet into a `StateEvent` and passes it to the `AppStateManager` for processing.
6.  **Event Throttling:** To avoid overwhelming the frontend with too many updates, it throttles the emission of events. By default, it emits events at most every 200 milliseconds.

## State Management: `state.rs`

The `AppStateManager` in `src-tauri/src/live/state.rs` is central to the live module's operation. It is responsible for:

-   **Maintaining Application State:** It holds the current state of the application, including information about the current encounter, players, entities, and combat events.
-   **Processing State Events:** The `handle_event` method takes a `StateEvent` and updates the application state accordingly. This is where the core logic for processing different types of game events resides.
-   **Emitting Events to Frontend:** The `update_and_emit_events` method is called periodically to send the latest data to the frontend, which then updates the live meter display.

## Event Manager: `event_manager.rs`

The `EventManager` is responsible for aggregating the processed data and preparing it for emission to the frontend. It works in conjunction with the `AppStateManager` to ensure that the frontend receives the correct data in a timely manner.

## Tauri Commands: `commands.rs`

The `live` module also exposes several Tauri commands that can be invoked from the frontend. These commands, defined in `src-tauri/src/live/commands.rs`, allow the frontend to control the behavior of the live meter. Some of the key commands include:

-   `enable_blur` and `disable_blur`: Toggle a blur effect on the live meter window.
-   `reset_encounter`: Manually reset the current encounter data.
-   `toggle_pause_encounter`: Pause or resume the recording of the encounter.
-   `get_player_skills`: Retrieve the skills used by a specific player.
-   `subscribe_player_skills` and `unsubscribe_player_skills`: Manage subscriptions for real-time updates of player skills.
-   `set_boss_only_dps`: A filter to show DPS only on boss targets.

These commands provide a powerful API for the frontend to interact with and control the real-time functionality of the backend.
