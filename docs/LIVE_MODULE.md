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
-   `get_encounter_phases`: Retrieve all phases (mob and boss segments) for a specific encounter.

These commands provide a powerful API for the frontend to interact with and control the real-time functionality of the backend.

## Phase Detection: `phase_detector.rs`

The phase detector (`src-tauri/src/live/phase_detector.rs`) is responsible for automatically identifying and separating distinct combat phases within encounters. This feature enables accurate metrics calculation and detailed analysis of multi-boss dungeons.

### Phase Types

The system recognizes two types of phases:

-   **Mob Phases**: Periods where the party fights non-boss enemies (trash mobs). Multiple mob phases can occur in a single encounter, separated by downtime.
-   **Boss Phases**: Periods where the party engages a boss enemy. Each boss gets its own phase, allowing for separate analysis of multi-boss encounters.

### Detection Logic

Phase detection operates through event-driven state transitions:

1.  **Combat Timeout**: A 15-second inactivity threshold detects downtime between combat segments. When no combat activity occurs for 15 seconds, the current phase ends.

2.  **Boss Detection**: When a high-HP entity matching boss characteristics takes damage, the system transitions to a boss phase. The phase tracks the specific boss entity ID for multi-boss support.

3.  **Mob Phase Splitting**: Mob phases are automatically split when combat gaps exceed the timeout threshold, creating distinct "Mob Pack 1", "Mob Pack 2", etc. segments.

4.  **Multi-Boss Support**: The system tracks multiple active bosses simultaneously using a `HashSet<i64>` of boss entity IDs. This enables proper handling of encounters with multiple bosses (sequential or simultaneous).

5.  **Boss Death Handling**: When a boss's HP reaches zero, the boss phase ends with a 'success' outcome. The system can then detect subsequent boss phases in the same encounter.

6.  **Adds/Minions**: Damage dealt to boss minions/adds is included in the active boss phase, not treated as separate mob phases.

### Phase Outcomes

Each phase can have one of three outcomes:

-   **success**: Phase completed successfully (boss killed or mobs cleared)
-   **wipe**: Party died during the phase
-   **unknown**: Phase ended due to timeout or incomplete data

### Downtime Exclusion

Phase detection enables accurate DPS/HPS calculations by excluding downtime (walking between pulls) from metrics:

-   **Active Combat Time**: Sum of all mob and boss phase durations
-   **DPS Calculation**: `Total Damage / Active Combat Time` (excludes downtime)
-   **HPS Calculation**: `Total Healing / Active Combat Time` (excludes downtime)

This provides significantly more accurate performance metrics compared to naive `Damage / Total Encounter Duration` calculations.

### Database Storage

Phase data is persisted in two tables:

-   **encounter_phases**: Stores phase metadata (type, timestamps, outcome)
-   **actor_phase_stats**: Stores per-actor performance statistics for each phase

### Frontend Integration

The frontend can retrieve phase data using the `get_encounter_phases` command and display phase-specific statistics. The PhaseSelector component (`src/lib/components/PhaseSelector.svelte`) provides a user interface for navigating between phases and viewing segment-specific metrics.
