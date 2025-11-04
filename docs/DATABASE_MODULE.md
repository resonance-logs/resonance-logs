# Database Module

The `database` module is responsible for all data persistence in the Resonance Logs application. It uses `diesel`, a powerful ORM and query builder for Rust, to interact with a local SQLite database. This module is designed to be efficient and robust, handling database operations in a background thread to avoid blocking the main application thread.

## Database Initialization: `mod.rs`

The `init_and_spawn_writer` function in `src-tauri/src/database/mod.rs` is the entry point for the database module. It is called during the application's setup in `lib.rs` and performs the following crucial tasks:

1.  **Database Path:** It determines the path for the SQLite database file, typically located in the user's local data directory.
2.  **Connection Pool:** It creates a connection pool using `diesel::r2d2`, which manages a set of database connections to be shared across threads. This is essential for handling concurrent database access from the main thread (for reading data) and the background writer thread.
3.  **Migrations:** It runs any pending database migrations using `diesel_migrations`. This ensures that the database schema is always up-to-date with the latest version of the application. The migrations are embedded directly into the application binary.
4.  **Background Writer Thread:** It spawns a dedicated background thread to handle all database write operations. This is a key design choice to prevent database writes from blocking the main application thread, which could lead to a sluggish user interface.

## Background Writer and Task Queue

The background writer thread operates on a task queue, which is implemented using a `tokio::sync::mpsc` channel. When other parts of the application need to write data to the database, they enqueue a `DbTask` into this channel.

The `DbTask` enum defines all possible database write operations, such as:

-   `BeginEncounter`: Starts a new encounter.
-   `EndEncounter`: Ends the current encounter.
-   `UpsertEntity`: Inserts or updates an entity (e.g., a player).
-   `InsertDamageEvent`: Records a damage event.
-   `InsertHealEvent`: Records a heal event.

The background thread continuously dequeues tasks from the channel and processes them in batches. This batching approach improves performance by reducing the number of individual database transactions.

## Data Materialization and Pruning

At the end of an encounter, the background thread performs several data materialization and pruning steps:

-   **Materialize Skill Stats:** It aggregates the raw damage and heal events into summary tables (`damage_skill_stats` and `heal_skill_stats`). This pre-calculation makes it much faster to query and display statistics in the frontend.
-   **Materialize Encounter Bosses:** It aggregates information about the bosses encountered during the fight.
-   **Prune Raw Events:** After the data has been materialized, the raw event data is deleted from the database to save space.

This process ensures that the database remains relatively small and efficient, even after recording many encounters.

## Tauri Commands: `commands.rs`

The `database` module exposes a rich set of Tauri commands in `src-tauri/src/database/commands.rs` that allow the frontend to query the database. These commands are used to populate the encounter history, player statistics, and other views in the application.

Some of the key commands include:

-   `get_recent_encounters`: Retrieves a list of recent encounters.
-   `get_encounter_by_id`: Fetches the details of a specific encounter.
-   `get_encounter_actor_stats`: Gets the statistics for all actors (players and monsters) in an encounter.
-   `get_player_names_filtered`: Provides a list of player names for filtering.
-   `delete_encounter`: Allows the user to delete an encounter from the database.

These commands provide a clean and secure interface for the frontend to access the stored data without having to worry about the underlying database implementation.
