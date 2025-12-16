pub mod commands;
pub mod models;
pub mod schema;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tokio::sync::mpsc::{self, Sender};

use crate::database::models as m;
use crate::database::schema as sch;
use serde_json::json;

/// A type alias for a database connection pool.
#[allow(dead_code)]
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// A constant holding the embedded database migrations.
pub const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();

// Sentinel value for unknown skill IDs in aggregated stats
const UNKNOWN_SKILL_ID_SENTINEL: i32 = -1;

static DB_SENDER: Lazy<Mutex<Option<Sender<DbTask>>>> = Lazy::new(|| Mutex::new(None));

/// Serializes DB initialization within the current process.
static DB_INIT_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// Serializes schema migrations within the current process.
///
/// Diesel's `run_pending_migrations` writes to the migrations table and can hold write locks.
/// If multiple threads invoke it concurrently (common during app startup when multiple Tauri
/// commands fire), SQLite will often return `database is locked`.
static DB_MIGRATIONS_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// Tracks whether migrations have been applied successfully in this process.
///
/// We intentionally do not cache failures permanently: transient SQLITE_BUSY/locked conditions
/// (e.g. antivirus scans, startup races) should be retryable later.
static DB_MIGRATIONS_APPLIED: AtomicBool = AtomicBool::new(false);

/// If DB initialization failed during startup (e.g. locked DB during migrations), we attempt
/// a best-effort background re-init when producers first try to enqueue tasks.
static DB_BACKGROUND_INIT_STARTED: AtomicBool = AtomicBool::new(false);

// Bounded channel capacity for DB task queue. Prevents unbounded memory
// growth when producers outpace the writer thread.
const BATCH_QUEUE_CAPACITY: usize = 5_000;

// Batch configuration: collect up to this many events or wait up to this many ms (whichever
// comes first) before flushing to the DB. Chosen by user: 50ms window.
const BATCH_MAX_EVENTS: usize = 100;
const BATCH_MAX_WAIT_MS: u64 = 50;

/// An enumeration of possible errors that can occur during database initialization.
#[derive(Debug, thiserror::Error)]
pub enum DbInitError {
    /// An error that occurred while creating the database connection pool.
    #[error("DB pool error: {0}")]
    Pool(String),
    /// An error that occurred while running database migrations.
    #[error("Migration error: {0}")]
    Migration(String),
}

/// Returns the current time in milliseconds since the Unix epoch.
///
/// # Returns
///
/// * `i64` - The current time in milliseconds.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Returns the default path to the database file.
///
/// # Returns
///
/// * `PathBuf` - The default path to the database file.
pub fn default_db_path() -> PathBuf {
    if let Some(mut dir) = dirs::data_local_dir() {
        dir.push("resonance-logs");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("resonance-logs.db")
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("resonance-logs.db")
    }
}

pub fn establish_connection() -> Result<SqliteConnection, diesel::ConnectionError> {
    let path = default_db_path();
    let opened = Instant::now();
    let mut conn = SqliteConnection::establish(&path.to_string_lossy())?;

    // Connection lifecycle logging: keep at debug to avoid spam at default levels.
    log::debug!(
        target: "app::db",
        "db_connection_opened path={} elapsed_ms={}",
        path.display(),
        opened.elapsed().as_millis()
    );

    apply_sqlite_pragmas(&mut conn);

    Ok(conn)
}

// Applies SQLite PRAGMAs that improve reliability and concurrency.
fn apply_sqlite_pragmas(conn: &mut SqliteConnection) {
    // Use a busy timeout so transient locks don't immediately fail writes/migrations.
    // Keep this comfortably above our migration retry window to reduce spurious failures.
    let _ = diesel::sql_query("PRAGMA busy_timeout=30000;").execute(conn);
    // WAL improves concurrent read/write behavior.
    let _ = diesel::sql_query("PRAGMA journal_mode=WAL;").execute(conn);
    // Reasonable durability/perf balance for app telemetry/logs.
    let _ = diesel::sql_query("PRAGMA synchronous=NORMAL;").execute(conn);
    // Enforce foreign keys.
    let _ = diesel::sql_query("PRAGMA foreign_keys=ON;").execute(conn);
}

// Ensures pool connections always get the same PRAGMA configuration.
#[derive(Debug)]
struct SqliteConnectionCustomizer;

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        apply_sqlite_pragmas(conn);
        Ok(())
    }
}

/// Ensures that the parent directory of a given path exists.
///
/// # Arguments
///
/// * `path` - The path to ensure the parent directory of.
///
/// # Returns
///
/// * `std::io::Result<()>` - An empty result indicating success or failure.
pub fn ensure_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Initializes the database and spawns a background writer thread.
///
/// # Returns
///
/// * `Result<(), DbInitError>` - An empty result indicating success or failure.
pub fn init_and_spawn_writer() -> Result<(), DbInitError> {
    let db_init_span = tracing::info_span!(target: "app::db", "db_init");
    let _db_init_guard = db_init_span.enter();

    // Prevent concurrent init attempts within the same process.
    let _single_flight = DB_INIT_MUTEX.lock();

    // If already initialized, do nothing.
    if DB_SENDER.lock().is_some() {
        log::debug!(target: "app::db", "db already initialized; skipping init");
        return Ok(());
    }

    let db_path = default_db_path();
    log::info!(target: "app::db", "db_path={}", db_path.display());

    if let Err(e) = ensure_parent_dir(&db_path) {
        return Err(DbInitError::Pool(format!("failed to create dir: {e}")));
    }

    let manager = ConnectionManager::<SqliteConnection>::new(db_path.to_string_lossy().to_string());
    // Increase connection pool size to reduce contention for DB connections
    // under concurrent UI reads + writer work. Tuned to 8 as a reasonable default (probably)
    let pool = Pool::builder()
        .max_size(8)
        .connection_customizer(Box::new(SqliteConnectionCustomizer))
        .build(manager)
        .map_err(|e| {
        let err_msg = format!("DB pool build error: {}", e);
        log::error!(target: "app::db", "{}", err_msg);
        DbInitError::Pool(e.to_string())
    })?;

    log::info!(
        target: "app::db",
        "db_pool_ready max_size={} queue_capacity={} batch_max_events={} batch_max_wait_ms={}",
        8,
        BATCH_QUEUE_CAPACITY,
        BATCH_MAX_EVENTS,
        BATCH_MAX_WAIT_MS
    );

    // Run migrations once (serialized across threads within this process)
    {
        let mut conn = pool.get().map_err(|e| DbInitError::Pool(e.to_string()))?;
        let mig_started = Instant::now();
        // Apply PRAGMAs before migrations since they affect lock handling.
        apply_sqlite_pragmas(&mut conn);

        let applied = ensure_migrations_on_conn_with_retry(&mut conn, Duration::from_secs(60))
            .map_err(|e| {
            let err_msg = format!("DB migration error: {}", e);
            log::error!(target: "app::db", "{}", err_msg);
            DbInitError::Migration(e)
        })?;
        log::info!(
            target: "app::db",
            "migrations_applied count={} elapsed_ms={}",
            applied,
            mig_started.elapsed().as_millis()
        );

        // Re-apply pragmas post-migration in case any migration changed them.
        apply_sqlite_pragmas(&mut conn);
    }

    // Spawn writer worker using a bounded channel to avoid unbounded memory
    // growth if producers are faster than the writer. Tasks will be dropped
    // (with a warning) when the queue is full to protect process memory.
    let (tx, mut rx) = mpsc::channel::<DbTask>(BATCH_QUEUE_CAPACITY);
    {
        let mut guard = DB_SENDER.lock();
        *guard = Some(tx.clone());
    }

    std::thread::spawn(move || {
        let writer_span = tracing::info_span!(target: "app::db", "db_writer_thread");
        let _writer_guard = writer_span.enter();

        log::info!(
            target: "app::db",
            "db_writer_thread_started batch_max_events={} batch_max_wait_ms={}",
            BATCH_MAX_EVENTS,
            BATCH_MAX_WAIT_MS
        );

        let mut current_encounter_id: Option<i32> = None;
        let mut current_encounter_start_ms: Option<i64> = None;
        loop {
            // Block until we receive the first task
            let first = rx.blocking_recv();
            let Some(first) = first else {
                log::info!(target: "app::db", "db_writer_thread_stopping (channel closed)");
                break;
            };

            // Collect a batch: include the first task, then try to drain more
            // tasks for up to BATCH_MAX_WAIT_MS or until BATCH_MAX_EVENTS.
            let mut tasks = Vec::with_capacity(BATCH_MAX_EVENTS);
            tasks.push(first);
            let start = Instant::now();
            while tasks.len() < BATCH_MAX_EVENTS
                && start.elapsed() < Duration::from_millis(BATCH_MAX_WAIT_MS)
            {
                match rx.try_recv() {
                    Ok(t) => tasks.push(t),
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                        // No immediate task available; sleep a little to avoid busy-looping
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
                }
            }

            // Acquire a connection and attempt to execute the batch in a single
            // transaction for better throughput. If the batch transaction fails,
            // fall back to processing tasks individually so we don't lose work.
            let mut conn = match pool.get() {
                Ok(c) => c,
                Err(e) => {
                    log::error!(target: "app::db", "DB get conn: {e}");
                    continue;
                }
            };

            let tasks_len = tasks.len();

            // Trace-level batch logging (can be enabled via RES_LOG) to avoid noisy defaults.
            log::trace!(target: "app::db", "db_batch_begin tasks={}", tasks_len);

            // Snapshot the current encounter state before processing the batch.
            // If the batch fails, we must restore this state because any changes (like clearing the ID)
            // made during the failed transaction are semantically rolled back, but the Rust variables
            // would otherwise remain changed.
            let pre_batch_encounter_id = current_encounter_id;
            let pre_batch_encounter_start = current_encounter_start_ms;

            let batch_result = conn.transaction::<(), diesel::result::Error, _>(|tx_conn| {
                for task in tasks.iter().cloned() {
                    handle_task(
                        tx_conn,
                        task,
                        &mut current_encounter_id,
                        &mut current_encounter_start_ms,
                    )
                    .map_err(|e| {
                        log::error!(target: "app::db", "DB task in batch failed: {}", e);
                        diesel::result::Error::RollbackTransaction
                    })?;
                }
                Ok(())
            });

            if let Err(e) = batch_result {
                log::error!(
                    target: "app::db",
                    "Batch transaction failed: {:?}. Falling back to per-task execution.",
                    e
                );

                // Restore the encounter state to what it was before the batch attempted to run.
                // This correctly handles cases where:
                // 1. BeginEncounter was in a PREVIOUS batch (so current_encounter_id is valid).
                // 2. BeginEncounter was in THIS batch (so it should be None after rollback).
                // 3. EndEncounter was in THIS batch (so it should be restored to the valid ID).
                current_encounter_id = pre_batch_encounter_id;
                current_encounter_start_ms = pre_batch_encounter_start;

                // Try to process tasks individually; this uses the same connection
                // (and therefore the same transaction semantics as earlier).
                // Important: Process BeginEncounter tasks first to ensure encounters exist
                // before damage/heal events that reference them.
                let mut begin_encounter_tasks = Vec::new();
                let mut other_tasks = Vec::new();
                for task in tasks {
                    if matches!(task, DbTask::BeginEncounter { .. }) {
                        begin_encounter_tasks.push(task);
                    } else {
                        other_tasks.push(task);
                    }
                }

                // Process BeginEncounter tasks first
                for task in begin_encounter_tasks {
                    if let Err(e) = handle_task(
                        &mut conn,
                        task,
                        &mut current_encounter_id,
                        &mut current_encounter_start_ms,
                    ) {
                        log::error!(
                            target: "app::db",
                            "DB task error (fallback - BeginEncounter): {}",
                            e
                        );
                    }
                }

                // Then process all other tasks
                for task in other_tasks {
                    if let Err(e) = handle_task(
                        &mut conn,
                        task,
                        &mut current_encounter_id,
                        &mut current_encounter_start_ms,
                    ) {
                        log::error!(target: "app::db", "DB task error (fallback): {}", e);
                    }
                }
            }

            log::trace!(
                target: "app::db",
                "db_batch_end tasks={} encounter_active={}",
                tasks_len,
                current_encounter_id.is_some()
            );
        }

        log::info!(target: "app::db", "db_writer_thread_exited");
    });

    Ok(())
}

/// Runs pending database migrations.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result indicating success or failure.
fn run_migrations(conn: &mut SqliteConnection) -> Result<usize, String> {
    conn.run_pending_migrations(MIGRATIONS)
        .map(|applied| applied.len())
        .map_err(|e| e.to_string())
}

/// Ensures pending migrations are applied exactly once per process.
pub fn ensure_migrations_on_conn(conn: &mut SqliteConnection) -> Result<usize, String> {
    ensure_migrations_on_conn_with_retry(conn, Duration::from_secs(60))
}

fn ensure_migrations_on_conn_with_retry(
    conn: &mut SqliteConnection,
    max_total_wait: Duration,
) -> Result<usize, String> {
    if DB_MIGRATIONS_APPLIED.load(Ordering::Acquire) {
        return Ok(0);
    }

    let _guard = DB_MIGRATIONS_MUTEX.lock();

    if DB_MIGRATIONS_APPLIED.load(Ordering::Acquire) {
        return Ok(0);
    }

    // Re-apply pragmas on the caller's connection before migrating.
    apply_sqlite_pragmas(conn);

    let applied = run_migrations_with_retry(conn, max_total_wait)?;
    DB_MIGRATIONS_APPLIED.store(true, Ordering::Release);
    Ok(applied)
}

// Runs pending database migrations with retries when SQLite is temporarily busy.
fn run_migrations_with_retry(
    conn: &mut SqliteConnection,
    max_total_wait: Duration,
) -> Result<usize, String> {
    let start = Instant::now();
    let mut attempt: u32 = 0;
    let mut backoff = Duration::from_millis(100);

    loop {
        match run_migrations(conn) {
            Ok(applied) => return Ok(applied),
            Err(e) => {
                let msg = e.to_lowercase();
                let is_locked = msg.contains("database is locked")
                    || msg.contains("sqlite_busy")
                    || msg.contains("busy")
                    || msg.contains("locked");

                if !is_locked {
                    return Err(e);
                }

                if start.elapsed() >= max_total_wait {
                    return Err(e);
                }

                attempt = attempt.saturating_add(1);
                log::warn!(
                    target: "app::db",
                    "db_migration_busy retry={} sleep_ms={} elapsed_ms={} err={}",
                    attempt,
                    backoff.as_millis(),
                    start.elapsed().as_millis(),
                    e
                );

                std::thread::sleep(backoff);
                // Exponential backoff with a cap.
                backoff = std::cmp::min(backoff.saturating_mul(2), Duration::from_secs(2));
            }
        }
    }
}

/// An enumeration of possible database tasks.
#[derive(Debug, Clone)]
pub enum DbTask {
    /// A task to begin a new encounter.
    BeginEncounter {
        started_at_ms: i64,
        local_player_id: Option<i64>,
        scene_id: Option<i32>,
        scene_name: Option<String>,
    },
    /// A task to end the current encounter.
    EndEncounter {
        ended_at_ms: i64,
        defeated_bosses: Option<Vec<String>>,
        /// Whether this encounter was manually reset by the user.
        is_manually_reset: bool,
        /// Per-player active damage time (ms) used for True DPS calculations.
        player_active_times: Vec<(i64, i64)>,
    },

    /// A task to end any encounters that never received an explicit end.
    EndAllActiveEncounters { ended_at_ms: i64 },

    /// A task to insert or update an entity.
    UpsertEntity {
        entity_id: i64,
        name: Option<String>,
        class_id: Option<i32>,
        class_spec: Option<i32>,
        ability_score: Option<i32>,
        level: Option<i32>,
        seen_at_ms: i64,
        attributes: Option<String>,
    },

    /// A task to insert or update detailed local player data.
    UpsertDetailedPlayerData {
        player_id: i64,
        last_seen_ms: i64,
        char_serialize_json: String,
        profession_list_json: Option<String>,
        talent_node_ids_json: Option<String>,
    },

    /// A task to insert a damage event.
    InsertDamageEvent {
        timestamp_ms: i64,
        attacker_id: i64,
        defender_id: Option<i64>,
        monster_name: Option<String>,
        skill_id: Option<i32>,
        value: i64,
        is_crit: bool,
        is_lucky: bool,
        hp_loss: i64,
        shield_loss: i64,
        defender_max_hp: Option<i64>,
        is_boss: bool,
        attempt_index: Option<i32>,
    },

    /// A task to insert a heal event.
    InsertHealEvent {
        timestamp_ms: i64,
        healer_id: i64,
        target_id: Option<i64>,
        skill_id: Option<i32>,
        value: i64,
        is_crit: bool,
        is_lucky: bool,
        attempt_index: Option<i32>,
    },
    /// A task to insert a death event.
    InsertDeathEvent {
        timestamp_ms: i64,
        actor_id: i64,
        killer_id: Option<i64>,
        skill_id: Option<i32>,
        is_local_player: bool,
        attempt_index: Option<i32>,
    },
    /// A task to record a revive (increments per-actor revive counter in actor_encounter_stats).
    InsertReviveEvent {
        timestamp_ms: i64,
        actor_id: i64,
        is_local_player: bool,
        attempt_index: Option<i32>,
    },

    /// A task to begin a new attempt within the current encounter.
    BeginAttempt {
        attempt_index: i32,
        started_at_ms: i64,
        reason: String,
        boss_hp_start: Option<i64>,
    },

    /// A task to end the current attempt.
    EndAttempt {
        attempt_index: i32,
        ended_at_ms: i64,
        boss_hp_end: Option<i64>,
        total_deaths: i32,
    },

    /// A task to insert a dungeon segment.
    InsertDungeonSegment {
        segment_type: String,
        boss_entity_id: Option<i64>,
        boss_monster_type_id: Option<i64>,
        boss_name: Option<String>,
        started_at_ms: i64,
        ended_at_ms: Option<i64>,
        total_damage: i64,
        hit_count: i64,
    },

    /// A task to save buff data for the current encounter.
    SaveBuffs {
        /// Map of (entity_id, buff_id) -> JSON-serialized events array.
        buffs: Vec<(i64, i32, String)>,
    },
}

/// Enqueues a database task to be processed by the background writer thread.
///
/// # Arguments
///
/// * `task` - The `DbTask` to enqueue.
pub fn enqueue(task: DbTask) {
    // Fire-and-forget for normal cases, but don't silently drop critical tasks
    // when the queue is saturated—block briefly to preserve encounter data.
    let guard = DB_SENDER.lock();
    if let Some(tx) = guard.as_ref() {
        if let Err(e) = tx.try_send(task.clone()) {
            log::warn!(
                "DB queue full or closed ({}); retrying with blocking_send to avoid data loss",
                e
            );
            if let Err(send_err) = tx.blocking_send(task) {
                log::error!("Failed to enqueue DB task after retry: {}", send_err);
            }
        }
    } else {
        // If init failed during startup due to a transient lock, kick off a background init.
        // We intentionally don't block the producer thread here.
        if !DB_BACKGROUND_INIT_STARTED.swap(true, Ordering::Relaxed) {
            std::thread::spawn(|| {
                let started = Instant::now();
                match init_and_spawn_writer() {
                    Ok(()) => log::info!(
                        target: "app::db",
                        "db_background_init_succeeded elapsed_ms={}",
                        started.elapsed().as_millis()
                    ),
                    Err(e) => log::warn!(
                        target: "app::db",
                        "db_background_init_failed elapsed_ms={} err={}",
                        started.elapsed().as_millis(),
                        e
                    ),
                }
            });
        }

        // Use an atomic flag to prevent spamming the log.
        static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            log::warn!(
                "DB queue not initialized; dropping database task (suppressing further warnings)"
            );
            eprintln!("DB queue not initialized; dropping database task");
        }
    }
}

fn finalize_encounter(
    conn: &mut SqliteConnection,
    encounter_id: i32,
    ended_at_ms: i64,
    defeated_bosses: Option<Vec<String>>,
    is_manually_reset: bool,
    player_active_times: Vec<(i64, i64)>,
) -> Result<(), String> {
    use sch::encounters::dsl as e;

    diesel::update(e::encounters.filter(e::id.eq(encounter_id)))
        .set((
            e::ended_at_ms.eq(ended_at_ms),
            e::is_manually_reset.eq(if is_manually_reset { 1 } else { 0 }),
        ))
        .execute(conn)
        .map_err(|er| er.to_string())?;

    let started_at_ms: i64 = e::encounters
        .filter(e::id.eq(encounter_id))
        .select(e::started_at_ms)
        .first::<i64>(conn)
        .map_err(|er| er.to_string())?;

    let mut duration_secs = 1.0_f64;
    if ended_at_ms > started_at_ms {
        let computed = ((ended_at_ms - started_at_ms) as f64) / 1000.0;
        if computed > 1.0 {
            duration_secs = computed;
        }
    }

    if !player_active_times.is_empty() {
        for (actor_id, active_ms) in player_active_times {
            diesel::sql_query(
                "UPDATE actor_encounter_stats
                 SET active_dmg_time_ms = ?3
                 WHERE encounter_id = ?1 AND actor_id = ?2",
            )
            .bind::<diesel::sql_types::Integer, _>(encounter_id)
            .bind::<diesel::sql_types::BigInt, _>(actor_id)
            .bind::<diesel::sql_types::BigInt, _>(active_ms)
            .execute(conn)
            .ok();
        }
    }

    diesel::sql_query(
        "UPDATE actor_encounter_stats
         SET duration = ?2,
             dps = CASE WHEN ?2 > 0 THEN damage_dealt * 1.0 / ?2 ELSE 0 END,
             tdps = CASE WHEN active_dmg_time_ms > 0 THEN damage_dealt * 1000.0 / active_dmg_time_ms ELSE 0 END
         WHERE encounter_id = ?1 AND is_player = 1",
    )
    .bind::<diesel::sql_types::Integer, _>(encounter_id)
    .bind::<diesel::sql_types::Double, _>(duration_secs)
    .execute(conn)
    .map_err(|er| er.to_string())?;

    diesel::update(e::encounters.filter(e::id.eq(encounter_id)))
        .set((
            e::duration.eq(duration_secs),
            e::ended_at_ms.eq(ended_at_ms),
        ))
        .execute(conn)
        .map_err(|er| er.to_string())?;

    // Update snapshot fields from entities table for any NULL values
    // This ensures we capture the final state at encounter end
    diesel::sql_query(
        "UPDATE actor_encounter_stats
         SET name = COALESCE(NULLIF(name, ''), (SELECT name FROM entities WHERE entity_id = actor_encounter_stats.actor_id)),
            class_id = COALESCE(NULLIF(class_id, 0), NULLIF((SELECT class_id FROM entities WHERE entity_id = actor_encounter_stats.actor_id), 0)),
            class_spec = COALESCE(NULLIF(class_spec, 0), NULLIF((SELECT class_spec FROM entities WHERE entity_id = actor_encounter_stats.actor_id), 0)),
             ability_score = COALESCE(NULLIF(ability_score, 0), NULLIF((SELECT ability_score FROM entities WHERE entity_id = actor_encounter_stats.actor_id), 0)),
             level = COALESCE(NULLIF(level, 0), NULLIF((SELECT level FROM entities WHERE entity_id = actor_encounter_stats.actor_id), 0)),
             attributes = COALESCE(NULLIF(attributes, ''), (SELECT attributes FROM entities WHERE entity_id = actor_encounter_stats.actor_id))
         WHERE encounter_id = ?1
          AND (name IS NULL
               OR name = ''
               OR class_id IS NULL
               OR class_id = 0
               OR class_spec IS NULL
               OR class_spec = 0
               OR ability_score IS NULL
               OR ability_score = 0
               OR level IS NULL
               OR level = 0
               OR attributes IS NULL
               OR attributes = '')"
    )
    .bind::<diesel::sql_types::Integer, _>(encounter_id)
    .execute(conn)
    .map_err(|er| er.to_string())?;

    // Aggregate damage events into damage_skill_stats and encounter_bosses
    materialize_damage_skill_stats(conn, encounter_id)?;
    materialize_encounter_bosses(conn, encounter_id)?;

    // If any defeated boss names were provided, mark them in encounter_bosses
    if let Some(names) = defeated_bosses {
        for name in names {
            diesel::sql_query(
                "UPDATE encounter_bosses SET is_defeated = 1 WHERE encounter_id = ?1 AND monster_name = ?2",
            )
            .bind::<diesel::sql_types::Integer, _>(encounter_id)
            .bind::<diesel::sql_types::Text, _>(&name)
            .execute(conn)
            .ok();
        }
    }

    // Aggregate heal events into heal_skill_stats
    materialize_heal_skill_stats(conn, encounter_id)?;

    // Delete raw events for this encounter to save space
    prune_encounter_events(conn, encounter_id)?;

    Ok(())
}

/// Handles a single database task.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
/// * `task` - The `DbTask` to handle.
/// * `current_encounter_id` - A mutable reference to the current encounter ID.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result indicating success or failure.
fn handle_task(
    conn: &mut SqliteConnection,
    task: DbTask,
    current_encounter_id: &mut Option<i32>,
    current_encounter_start_ms: &mut Option<i64>,
) -> Result<(), String> {
    match task {
        DbTask::BeginEncounter {
            started_at_ms,
            local_player_id,
            scene_id,
            scene_name,
        } => {
            if current_encounter_id.is_some() {
                return Ok(());
            }
            use sch::encounters::dsl as e;
            let new_enc = m::NewEncounter {
                started_at_ms,
                ended_at_ms: None,
                local_player_id,
                total_dmg: Some(0),
                total_heal: Some(0),
                scene_id,
                scene_name,
                duration: 0.0,
            };
            diesel::insert_into(e::encounters)
                .values(&new_enc)
                .execute(conn)
                .map_err(|er| er.to_string())?;
            let id: i32 = e::encounters
                .order(e::id.desc())
                .select(e::id)
                .first::<i32>(conn)
                .map_err(|e| e.to_string())?;
            *current_encounter_id = Some(id);
            *current_encounter_start_ms = Some(started_at_ms);
        }
        DbTask::EndEncounter {
            ended_at_ms,
            defeated_bosses,
            is_manually_reset,
            player_active_times,
        } => {
            if let Some(id) = current_encounter_id.take() {
                finalize_encounter(
                    conn,
                    id,
                    ended_at_ms,
                    defeated_bosses,
                    is_manually_reset,
                    player_active_times,
                )?;
            }
            *current_encounter_start_ms = None;
        }
        DbTask::EndAllActiveEncounters { ended_at_ms } => {
            use sch::encounters::dsl as e;
            let open_encounters: Vec<i32> = e::encounters
                .filter(e::ended_at_ms.is_null())
                .select(e::id)
                .load::<i32>(conn)
                .map_err(|er| er.to_string())?;

            for encounter_id in open_encounters {
                finalize_encounter(conn, encounter_id, ended_at_ms, None, false, Vec::new())?;
            }

            *current_encounter_id = None;
            *current_encounter_start_ms = None;
        }
        DbTask::UpsertEntity {
            entity_id,
            name,
            class_id,
            class_spec,
            ability_score,
            level,
            seen_at_ms,
            attributes,
        } => {
            let normalized_class_id = class_id.filter(|&cid| cid > 0);
            let normalized_class_spec = class_spec.filter(|&cs| cs > 0);
            let normalized_ability_score = ability_score.filter(|&score| score > 0);
            let normalized_level = level.filter(|&lvl| lvl > 0);

            use sch::entities::dsl as en;
            let exists: Option<i64> = en::entities
                .select(en::entity_id)
                .filter(en::entity_id.eq(entity_id))
                .first::<i64>(conn)
                .optional()
                .map_err(|e| e.to_string())?;
            if exists.is_some() {
                let update = m::UpdateEntity {
                    name: name
                        .as_deref()
                        .and_then(|s| (!s.trim().is_empty()).then_some(s)),
                    class_id: normalized_class_id,
                    class_spec: normalized_class_spec,
                    ability_score: normalized_ability_score,
                    level: normalized_level,
                    last_seen_ms: Some(seen_at_ms),
                    attributes: attributes.as_deref(),
                };
                diesel::update(en::entities.filter(en::entity_id.eq(entity_id)))
                    .set(&update)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
            } else {
                let insert = m::NewEntity {
                    entity_id,
                    name: name
                        .as_deref()
                        .and_then(|s| (!s.trim().is_empty()).then_some(s)),
                    class_id: normalized_class_id,
                    class_spec: normalized_class_spec,
                    ability_score: normalized_ability_score,
                    level: normalized_level,
                    first_seen_ms: Some(seen_at_ms),
                    last_seen_ms: Some(seen_at_ms),
                    attributes: attributes.as_deref(),
                };
                diesel::insert_into(en::entities)
                    .values(&insert)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
            }
        }
        DbTask::UpsertDetailedPlayerData {
            player_id,
            last_seen_ms,
            char_serialize_json,
            profession_list_json,
            talent_node_ids_json,
        } => {
            use sch::detailed_playerdata::dsl as dp;

            let exists: Option<i64> = dp::detailed_playerdata
                .select(dp::player_id)
                .filter(dp::player_id.eq(player_id))
                .first::<i64>(conn)
                .optional()
                .map_err(|e| e.to_string())?;

            let char_serialize_str = char_serialize_json.as_str();
            let profession_list_str = profession_list_json.as_deref();
            let talent_node_ids_str = talent_node_ids_json.as_deref();

            if exists.is_some() {
                let update = m::UpdateDetailedPlayerData {
                    last_seen_ms,
                    char_serialize_json: char_serialize_str,
                    profession_list_json: profession_list_str,
                    talent_node_ids_json: talent_node_ids_str,
                };
                diesel::update(dp::detailed_playerdata.filter(dp::player_id.eq(player_id)))
                    .set(&update)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
            } else {
                let insert = m::NewDetailedPlayerData {
                    player_id,
                    last_seen_ms,
                    char_serialize_json: char_serialize_str,
                    profession_list_json: profession_list_str,
                    talent_node_ids_json: talent_node_ids_str,
                };
                diesel::insert_into(dp::detailed_playerdata)
                    .values(&insert)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
            }
        }
        DbTask::InsertDamageEvent {
            timestamp_ms,
            attacker_id,
            defender_id,
            monster_name,
            skill_id,
            value,
            is_crit,
            is_lucky,
            hp_loss,
            shield_loss,
            defender_max_hp,
            is_boss,
            attempt_index,
        } => {
            // Raw per-event storage has been removed. We increment aggregate tables directly.
            if let Some(enc_id) = *current_encounter_id {
                // increment encounter totals
                diesel::sql_query(
                    "UPDATE encounters SET total_dmg = COALESCE(total_dmg,0) + ?1 WHERE id = ?2",
                )
                .bind::<diesel::sql_types::BigInt, _>(value)
                .bind::<diesel::sql_types::Integer, _>(enc_id)
                .execute(conn)
                .ok();

                // Materialize per-actor stats: attacker damage_dealt; defender damage_taken
                upsert_stats_add_damage_dealt(
                    conn,
                    enc_id,
                    attacker_id,
                    value,
                    is_crit,
                    is_lucky,
                    is_boss,
                )?;

                if let Some(def_id) = defender_id {
                    if let Some(attacker_type) = get_entity_type(conn, attacker_id)? {
                        if attacker_type
                            != (blueprotobuf_lib::blueprotobuf::EEntityType::EntChar as i32)
                        {
                            upsert_stats_add_damage_taken(
                                conn, enc_id, def_id, value, is_crit, is_lucky,
                            )?;
                        }
                    } else {
                        upsert_stats_add_damage_taken(
                            conn, enc_id, def_id, value, is_crit, is_lucky,
                        )?;
                    }
                }

                // Upsert per-skill aggregated stats into damage_skill_stats
                let skill_id_val: i32 = skill_id.unwrap_or(UNKNOWN_SKILL_ID_SENTINEL);
                let crit_i: i32 = if is_crit { 1 } else { 0 };
                let lucky_i: i32 = if is_lucky { 1 } else { 0 };

                let hit_detail = json!({
                    "timestamp": timestamp_ms,
                    "ms_from_start": timestamp_ms - current_encounter_start_ms.unwrap_or(timestamp_ms),
                    "damage": value,
                    "crit": is_crit,
                    "lucky": is_lucky,
                    "hp_loss": hp_loss,
                    "shield_loss": shield_loss,
                    "is_boss": is_boss,
                    "attempt_index": attempt_index,
                });
                let hit_detail_str = hit_detail.to_string();

                diesel::sql_query(
                    "INSERT INTO damage_skill_stats (encounter_id, attacker_id, defender_id, skill_id, hits, total_value, crit_hits, lucky_hits, crit_total, lucky_total, hp_loss_total, shield_loss_total, hit_details, monster_name) \
                     VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, ?7, ?8, ?9, ?10, ?11, json_array(json(?13)), ?12) \
                     ON CONFLICT(encounter_id, attacker_id, defender_id, skill_id) DO UPDATE SET \
                         hits = hits + 1, total_value = total_value + excluded.total_value, \
                         crit_hits = crit_hits + excluded.crit_hits, lucky_hits = lucky_hits + excluded.lucky_hits, \
                         crit_total = crit_total + excluded.crit_total, lucky_total = lucky_total + excluded.lucky_total, \
                         hp_loss_total = hp_loss_total + excluded.hp_loss_total, shield_loss_total = shield_loss_total + excluded.shield_loss_total, \
                         hit_details = json_insert(hit_details, '$[#]', json(?13))",
                )
                .bind::<diesel::sql_types::Integer, _>(enc_id)
                .bind::<diesel::sql_types::BigInt, _>(attacker_id)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(defender_id)
                .bind::<diesel::sql_types::Integer, _>(skill_id_val)
                .bind::<diesel::sql_types::BigInt, _>(value)
                .bind::<diesel::sql_types::Integer, _>(crit_i)
                .bind::<diesel::sql_types::Integer, _>(lucky_i)
                .bind::<diesel::sql_types::BigInt, _>(if is_crit { value } else { 0 })
                .bind::<diesel::sql_types::BigInt, _>(if is_lucky { value } else { 0 })
                .bind::<diesel::sql_types::BigInt, _>(hp_loss)
                .bind::<diesel::sql_types::BigInt, _>(shield_loss)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(monster_name.clone())
                .bind::<diesel::sql_types::Text, _>(hit_detail_str)
                .execute(conn)
                .ok();

                // Maintain encounter_bosses incrementally for boss hits
                if is_boss {
                    if let Some(monname) = monster_name.clone() {
                        diesel::sql_query(
                            "INSERT INTO encounter_bosses (encounter_id, monster_name, hits, total_damage, max_hp, is_defeated) \
                             VALUES (?1, ?2, 1, ?3, ?4, 0) \
                             ON CONFLICT(encounter_id, monster_name) DO UPDATE SET \
                               hits = hits + 1, total_damage = total_damage + excluded.total_damage, \
                               max_hp = COALESCE(excluded.max_hp, max_hp)"
                        )
                        .bind::<diesel::sql_types::Integer, _>(enc_id)
                        .bind::<diesel::sql_types::Text, _>(&monname)
                        .bind::<diesel::sql_types::BigInt, _>(value)
                        .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(defender_max_hp)
                        .execute(conn)
                        .ok();
                    }
                }
            }
        }
        DbTask::InsertHealEvent {
            timestamp_ms,
            healer_id,
            target_id,
            skill_id,
            value,
            is_crit,
            is_lucky,
            attempt_index,
        } => {
            // Raw per-event storage removed — update aggregates directly.
            if let Some(enc_id) = *current_encounter_id {
                diesel::sql_query(
                    "UPDATE encounters SET total_heal = COALESCE(total_heal,0) + ?1 WHERE id = ?2",
                )
                .bind::<diesel::sql_types::BigInt, _>(value)
                .bind::<diesel::sql_types::Integer, _>(enc_id)
                .execute(conn)
                .ok();

                upsert_stats_add_heal_dealt(conn, enc_id, healer_id, value, is_crit, is_lucky)?;

                // Upsert into heal_skill_stats
                let skill_id_val: i32 = skill_id.unwrap_or(UNKNOWN_SKILL_ID_SENTINEL);
                let crit_i: i32 = if is_crit { 1 } else { 0 };
                let lucky_i: i32 = if is_lucky { 1 } else { 0 };

                let heal_detail = json!({
                    "timestamp": timestamp_ms,
                    "ms_from_start": timestamp_ms - current_encounter_start_ms.unwrap_or(timestamp_ms),
                    "heal": value,
                    "crit": is_crit,
                    "lucky": is_lucky,
                    "attempt_index": attempt_index,
                });
                let heal_detail_str = heal_detail.to_string();

                diesel::sql_query(
                    "INSERT INTO heal_skill_stats (encounter_id, healer_id, target_id, skill_id, hits, total_value, crit_hits, lucky_hits, crit_total, lucky_total, heal_details, monster_name) \
                     VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, ?7, ?8, ?9, json_array(json(?11)), ?10) \
                     ON CONFLICT(encounter_id, healer_id, target_id, skill_id) DO UPDATE SET \
                         hits = hits + 1, total_value = total_value + excluded.total_value, \
                         crit_hits = crit_hits + excluded.crit_hits, lucky_hits = lucky_hits + excluded.lucky_hits, \
                         crit_total = crit_total + excluded.crit_total, lucky_total = lucky_total + excluded.lucky_total, \
                         heal_details = json_insert(heal_details, '$[#]', json(?11))",
                )
                .bind::<diesel::sql_types::Integer, _>(enc_id)
                .bind::<diesel::sql_types::BigInt, _>(healer_id)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(target_id)
                .bind::<diesel::sql_types::Integer, _>(skill_id_val)
                .bind::<diesel::sql_types::BigInt, _>(value)
                .bind::<diesel::sql_types::Integer, _>(crit_i)
                .bind::<diesel::sql_types::Integer, _>(lucky_i)
                .bind::<diesel::sql_types::BigInt, _>(if is_crit { value } else { 0 })
                .bind::<diesel::sql_types::BigInt, _>(if is_lucky { value } else { 0 })
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
                .bind::<diesel::sql_types::Text, _>(heal_detail_str)
                .execute(conn)
                .ok();
            }
        }
        DbTask::InsertDeathEvent {
            timestamp_ms,
            actor_id,
            killer_id,
            skill_id,
            is_local_player,
            attempt_index,
        } => {
            if let Some(enc_id) = *current_encounter_id {
                use sch::death_events::dsl as d;
                let ins = m::NewDeathEvent {
                    encounter_id: enc_id,
                    timestamp_ms,
                    actor_id,
                    killer_id,
                    skill_id,
                    is_local_player: if is_local_player { 1 } else { 0 },
                    attempt_index,
                };
                diesel::insert_into(d::death_events)
                    .values(&ins)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
            }
        }
        DbTask::InsertReviveEvent {
            timestamp_ms: _,
            actor_id,
            is_local_player: _,
            attempt_index: _,
        } => {
            if let Some(enc_id) = *current_encounter_id {
                // Increment per-actor revive counter
                upsert_stats_add_revive(conn, enc_id, actor_id)?;
            }
        }
        DbTask::BeginAttempt {
            attempt_index,
            started_at_ms,
            reason,
            boss_hp_start,
        } => {
            if let Some(enc_id) = *current_encounter_id {
                use sch::attempts::dsl as a;
                let ins = m::NewAttempt {
                    encounter_id: enc_id,
                    attempt_index,
                    started_at_ms,
                    ended_at_ms: None,
                    reason,
                    boss_hp_start,
                    boss_hp_end: None,
                    total_deaths: 0,
                };
                diesel::insert_into(a::attempts)
                    .values(&ins)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
            }
        }
        DbTask::EndAttempt {
            attempt_index,
            ended_at_ms,
            boss_hp_end,
            total_deaths,
        } => {
            if let Some(enc_id) = *current_encounter_id {
                use sch::attempts::dsl as a;
                diesel::update(
                    a::attempts
                        .filter(a::encounter_id.eq(enc_id))
                        .filter(a::attempt_index.eq(attempt_index)),
                )
                .set((
                    a::ended_at_ms.eq(Some(ended_at_ms)),
                    a::boss_hp_end.eq(boss_hp_end),
                    a::total_deaths.eq(total_deaths),
                ))
                .execute(conn)
                .map_err(|e| e.to_string())?;
            }
        }
        DbTask::InsertDungeonSegment {
            segment_type,
            boss_entity_id,
            boss_monster_type_id,
            boss_name,
            started_at_ms,
            ended_at_ms,
            total_damage,
            hit_count,
        } => {
            let target_encounter_id = if let Some(enc_id) = *current_encounter_id {
                Some(enc_id)
            } else {
                find_encounter_id_for_segment(conn, started_at_ms)?
            };

            if let Some(enc_id) = target_encounter_id {
                use sch::dungeon_segments::dsl as ds;
                let new_segment = m::NewDungeonSegment {
                    encounter_id: enc_id,
                    segment_type: &segment_type,
                    boss_entity_id,
                    boss_monster_type_id,
                    boss_name: boss_name.as_deref(),
                    started_at_ms,
                    ended_at_ms,
                    total_damage,
                    hit_count,
                };
                diesel::insert_into(ds::dungeon_segments)
                    .values(&new_segment)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
            } else {
                log::warn!(
                    "Dropping dungeon segment {:?} ({:?}) – no matching encounter for start_ms {}",
                    segment_type,
                    boss_name,
                    started_at_ms
                );
            }
        }
        DbTask::SaveBuffs { buffs } => {
            if let Some(enc_id) = *current_encounter_id {
                for (entity_id, buff_id, events_json) in buffs {
                    diesel::sql_query(
                        "INSERT INTO buffs (encounter_id, entity_id, buff_id, events)
                         VALUES (?1, ?2, ?3, ?4)
                         ON CONFLICT(encounter_id, entity_id, buff_id) DO UPDATE SET
                             events = excluded.events",
                    )
                    .bind::<diesel::sql_types::Integer, _>(enc_id)
                    .bind::<diesel::sql_types::BigInt, _>(entity_id)
                    .bind::<diesel::sql_types::Integer, _>(buff_id)
                    .bind::<diesel::sql_types::Text, _>(&events_json)
                    .execute(conn)
                    .ok();
                }
            }
        }
    }
    Ok(())
}

/// Attempt to find the encounter id that was active when a dungeon segment started.
/// Falls back to the most recent encounter that started on or before the segment timestamp.
fn find_encounter_id_for_segment(
    conn: &mut SqliteConnection,
    segment_start_ms: i64,
) -> Result<Option<i32>, String> {
    use sch::encounters::dsl as e;

    e::encounters
        .select(e::id)
        .filter(e::started_at_ms.le(segment_start_ms))
        .order(e::started_at_ms.desc())
        .first::<i32>(conn)
        .optional()
        .map_err(|er| er.to_string())
}

/// Gets the entity type of a given entity ID.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
/// * `entity_id` - The ID of the entity.
///
/// # Returns
///
/// * `Result<Option<i32>, String>` - The entity type, or `None` if the entity is not found.
fn get_entity_type(conn: &mut SqliteConnection, entity_id: i64) -> Result<Option<i32>, String> {
    // Entities table no longer stores entity_type; we only persist players.
    // If an entity exists here, treat it as a player (EntChar); otherwise unknown.
    use sch::entities::dsl as en;
    let exists: Option<i64> = en::entities
        .select(en::entity_id)
        .filter(en::entity_id.eq(entity_id))
        .first::<i64>(conn)
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(exists.map(|_| blueprotobuf_lib::blueprotobuf::EEntityType::EntChar as i32))
}

/// Upserts actor encounter stats for damage dealt.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
/// * `encounter_id` - The ID of the encounter.
/// * `actor_id` - The ID of the actor.
/// * `value` - The amount of damage dealt.
/// * `is_crit` - Whether the damage was a critical hit.
/// * `is_lucky` - Whether the damage was a lucky hit.
/// * `is_boss` - Whether the target was a boss.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result indicating success or failure.
fn upsert_stats_add_damage_dealt(
    conn: &mut SqliteConnection,
    encounter_id: i32,
    actor_id: i64,
    value: i64,
    is_crit: bool,
    is_lucky: bool,
    is_boss: bool,
) -> Result<(), String> {
    let crit_hit = if is_crit { 1_i64 } else { 0_i64 };
    let lucky_hit = if is_lucky { 1_i64 } else { 0_i64 };
    let boss_hit = if is_boss { 1_i64 } else { 0_i64 };
    diesel::sql_query(
                        "INSERT INTO actor_encounter_stats (
                             encounter_id, actor_id, name, class_id, ability_score, level, is_player,
                             class_spec, is_local_player, attributes,
                             damage_dealt, hits_dealt, crit_hits_dealt, lucky_hits_dealt, crit_total_dealt, lucky_total_dealt,
                             boss_damage_dealt, boss_hits_dealt, boss_crit_hits_dealt, boss_lucky_hits_dealt, boss_crit_total_dealt, boss_lucky_total_dealt,
                             revives
                         ) VALUES (
                             ?1, ?2,
                             (SELECT name FROM entities WHERE entity_id = ?2),
                             NULLIF((SELECT class_id FROM entities WHERE entity_id = ?2), 0),
                             NULLIF((SELECT ability_score FROM entities WHERE entity_id = ?2), 0),
                             NULLIF((SELECT level FROM entities WHERE entity_id = ?2), 0),
                             CASE WHEN EXISTS(SELECT 1 FROM entities WHERE entity_id = ?2) THEN 1 ELSE 0 END,
                             NULLIF((SELECT class_spec FROM entities WHERE entity_id = ?2), 0),
                             CASE WHEN ?2 = (SELECT local_player_id FROM encounters WHERE id = ?1) THEN 1 ELSE 0 END,
                             (SELECT attributes FROM entities WHERE entity_id = ?2),
                             ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14
                         ) ON CONFLICT(encounter_id, actor_id) DO UPDATE SET
                        name = COALESCE(NULLIF(actor_encounter_stats.name, ''), (SELECT name FROM entities WHERE entity_id = excluded.actor_id)),
                            class_id = COALESCE(NULLIF(actor_encounter_stats.class_id, 0), NULLIF((SELECT class_id FROM entities WHERE entity_id = excluded.actor_id), 0)),
                            class_spec = COALESCE(NULLIF(actor_encounter_stats.class_spec, 0), NULLIF((SELECT class_spec FROM entities WHERE entity_id = excluded.actor_id), 0)),
                        ability_score = COALESCE(NULLIF(actor_encounter_stats.ability_score, 0), NULLIF((SELECT ability_score FROM entities WHERE entity_id = excluded.actor_id), 0)),
                        level = COALESCE(NULLIF(actor_encounter_stats.level, 0), NULLIF((SELECT level FROM entities WHERE entity_id = excluded.actor_id), 0)),
                        attributes = COALESCE(NULLIF(actor_encounter_stats.attributes, ''), (SELECT attributes FROM entities WHERE entity_id = excluded.actor_id)),
                         damage_dealt = damage_dealt + excluded.damage_dealt,
                         hits_dealt = hits_dealt + excluded.hits_dealt,
                         crit_hits_dealt = crit_hits_dealt + excluded.crit_hits_dealt,
                         lucky_hits_dealt = lucky_hits_dealt + excluded.lucky_hits_dealt,
                         crit_total_dealt = crit_total_dealt + excluded.crit_total_dealt,
                         lucky_total_dealt = lucky_total_dealt + excluded.lucky_total_dealt,
                         boss_damage_dealt = boss_damage_dealt + excluded.boss_damage_dealt,
                         boss_hits_dealt = boss_hits_dealt + excluded.boss_hits_dealt,
                         boss_crit_hits_dealt = boss_crit_hits_dealt + excluded.boss_crit_hits_dealt,
                         boss_lucky_hits_dealt = boss_lucky_hits_dealt + excluded.boss_lucky_hits_dealt,
                         boss_crit_total_dealt = boss_crit_total_dealt + excluded.boss_crit_total_dealt,
                         boss_lucky_total_dealt = boss_lucky_total_dealt + excluded.boss_lucky_total_dealt,
                         revives = revives + excluded.revives"
            )
    .bind::<diesel::sql_types::Integer, _>(encounter_id)
    .bind::<diesel::sql_types::BigInt, _>(actor_id)
    .bind::<diesel::sql_types::BigInt, _>(value)
    .bind::<diesel::sql_types::BigInt, _>(crit_hit)
    .bind::<diesel::sql_types::BigInt, _>(lucky_hit)
    .bind::<diesel::sql_types::BigInt, _>(if is_crit { value } else { 0 })
    .bind::<diesel::sql_types::BigInt, _>(if is_lucky { value } else { 0 })
    .bind::<diesel::sql_types::BigInt, _>(if is_boss { value } else { 0 })
    .bind::<diesel::sql_types::BigInt, _>(boss_hit)
    .bind::<diesel::sql_types::BigInt, _>(if is_boss && is_crit { 1 } else { 0 })
    .bind::<diesel::sql_types::BigInt, _>(if is_boss && is_lucky { 1 } else { 0 })
    .bind::<diesel::sql_types::BigInt, _>(if is_boss && is_crit { value } else { 0 })
    .bind::<diesel::sql_types::BigInt, _>(if is_boss && is_lucky { value } else { 0 })
    // revives column (no revive added for damage events)
    .bind::<diesel::sql_types::BigInt, _>(0_i64)
    .execute(conn)
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Upserts actor encounter stats for heal dealt.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
/// * `encounter_id` - The ID of the encounter.
/// * `actor_id` - The ID of the actor.
/// * `value` - The amount of heal dealt.
/// * `is_crit` - Whether the heal was a critical hit.
/// * `is_lucky` - Whether the heal was a lucky hit.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result indicating success or failure.
fn upsert_stats_add_heal_dealt(
    conn: &mut SqliteConnection,
    encounter_id: i32,
    actor_id: i64,
    value: i64,
    is_crit: bool,
    is_lucky: bool,
) -> Result<(), String> {
    let crit_hit = if is_crit { 1_i64 } else { 0_i64 };
    let lucky_hit = if is_lucky { 1_i64 } else { 0_i64 };
    diesel::sql_query(
                                    "INSERT INTO actor_encounter_stats (
                                             encounter_id, actor_id, name, class_id, ability_score, level, is_player,
                                             class_spec, is_local_player, attributes,
                                             heal_dealt, hits_heal, crit_hits_heal, lucky_hits_heal, crit_total_heal, lucky_total_heal
                                     ) VALUES (
                                             ?1, ?2,
                                             (SELECT name FROM entities WHERE entity_id = ?2),
                                             NULLIF((SELECT class_id FROM entities WHERE entity_id = ?2), 0),
                                             NULLIF((SELECT ability_score FROM entities WHERE entity_id = ?2), 0),
                                             NULLIF((SELECT level FROM entities WHERE entity_id = ?2), 0),
                                             CASE WHEN EXISTS(SELECT 1 FROM entities WHERE entity_id = ?2) THEN 1 ELSE 0 END,
                                             NULLIF((SELECT class_spec FROM entities WHERE entity_id = ?2), 0),
                                             CASE WHEN ?2 = (SELECT local_player_id FROM encounters WHERE id = ?1) THEN 1 ELSE 0 END,
                                             (SELECT attributes FROM entities WHERE entity_id = ?2),
                                             ?3, 1, ?4, ?5, ?6, ?7
                                     ) ON CONFLICT(encounter_id, actor_id) DO UPDATE SET
                                        name = COALESCE(NULLIF(actor_encounter_stats.name, ''), (SELECT name FROM entities WHERE entity_id = excluded.actor_id)),
                                            class_id = COALESCE(NULLIF(actor_encounter_stats.class_id, 0), NULLIF((SELECT class_id FROM entities WHERE entity_id = excluded.actor_id), 0)),
                                            class_spec = COALESCE(NULLIF(actor_encounter_stats.class_spec, 0), NULLIF((SELECT class_spec FROM entities WHERE entity_id = excluded.actor_id), 0)),
                                        ability_score = COALESCE(NULLIF(actor_encounter_stats.ability_score, 0), NULLIF((SELECT ability_score FROM entities WHERE entity_id = excluded.actor_id), 0)),
                                        level = COALESCE(NULLIF(actor_encounter_stats.level, 0), NULLIF((SELECT level FROM entities WHERE entity_id = excluded.actor_id), 0)),
                                        attributes = COALESCE(NULLIF(actor_encounter_stats.attributes, ''), (SELECT attributes FROM entities WHERE entity_id = excluded.actor_id)),
                                         heal_dealt = heal_dealt + excluded.heal_dealt,
                                         hits_heal = hits_heal + excluded.hits_heal,
                                         crit_hits_heal = crit_hits_heal + excluded.crit_hits_heal,
                                         lucky_hits_heal = lucky_hits_heal + excluded.lucky_hits_heal,
                                         crit_total_heal = crit_total_heal + excluded.crit_total_heal,
                                         lucky_total_heal = lucky_total_heal + excluded.lucky_total_heal"
                )
    .bind::<diesel::sql_types::Integer, _>(encounter_id)
    .bind::<diesel::sql_types::BigInt, _>(actor_id)
    .bind::<diesel::sql_types::BigInt, _>(value)
    .bind::<diesel::sql_types::BigInt, _>(crit_hit)
    .bind::<diesel::sql_types::BigInt, _>(lucky_hit)
    .bind::<diesel::sql_types::BigInt, _>(if is_crit { value } else { 0 })
    .bind::<diesel::sql_types::BigInt, _>(if is_lucky { value } else { 0 })
    .execute(conn)
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Upserts actor encounter stats for a revive event (increments revive counter).
fn upsert_stats_add_revive(
    conn: &mut SqliteConnection,
    encounter_id: i32,
    actor_id: i64,
) -> Result<(), String> {
    diesel::sql_query(
        "INSERT INTO actor_encounter_stats (
                 encounter_id, actor_id, name, class_id, ability_score, level, is_player,
                 class_spec, is_local_player, attributes,
                 revives
         ) VALUES (
                 ?1, ?2,
                 (SELECT name FROM entities WHERE entity_id = ?2),
                 NULLIF((SELECT class_id FROM entities WHERE entity_id = ?2), 0),
                 NULLIF((SELECT ability_score FROM entities WHERE entity_id = ?2), 0),
                 NULLIF((SELECT level FROM entities WHERE entity_id = ?2), 0),
                 CASE WHEN EXISTS(SELECT 1 FROM entities WHERE entity_id = ?2) THEN 1 ELSE 0 END,
                 NULLIF((SELECT class_spec FROM entities WHERE entity_id = ?2), 0),
                 CASE WHEN ?2 = (SELECT local_player_id FROM encounters WHERE id = ?1) THEN 1 ELSE 0 END,
                 (SELECT attributes FROM entities WHERE entity_id = ?2),
                 ?3
         ) ON CONFLICT(encounter_id, actor_id) DO UPDATE SET
            name = COALESCE(NULLIF(actor_encounter_stats.name, ''), (SELECT name FROM entities WHERE entity_id = excluded.actor_id)),
            class_id = COALESCE(NULLIF(actor_encounter_stats.class_id, 0), NULLIF((SELECT class_id FROM entities WHERE entity_id = excluded.actor_id), 0)),
            class_spec = COALESCE(NULLIF(actor_encounter_stats.class_spec, 0), NULLIF((SELECT class_spec FROM entities WHERE entity_id = excluded.actor_id), 0)),
            ability_score = COALESCE(NULLIF(actor_encounter_stats.ability_score, 0), NULLIF((SELECT ability_score FROM entities WHERE entity_id = excluded.actor_id), 0)),
            level = COALESCE(NULLIF(actor_encounter_stats.level, 0), NULLIF((SELECT level FROM entities WHERE entity_id = excluded.actor_id), 0)),
            attributes = COALESCE(NULLIF(actor_encounter_stats.attributes, ''), (SELECT attributes FROM entities WHERE entity_id = excluded.actor_id)),
            revives = revives + excluded.revives"
    )
    .bind::<diesel::sql_types::Integer, _>(encounter_id)
    .bind::<diesel::sql_types::BigInt, _>(actor_id)
    .bind::<diesel::sql_types::BigInt, _>(1_i64)
    .execute(conn)
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Upserts actor encounter stats for damage taken.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
/// * `encounter_id` - The ID of the encounter.
/// * `actor_id` - The ID of the actor.
/// * `value` - The amount of damage taken.
/// * `is_crit` - Whether the damage was a critical hit.
/// * `is_lucky` - Whether the damage was a lucky hit.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result indicating success or failure.
fn upsert_stats_add_damage_taken(
    conn: &mut SqliteConnection,
    encounter_id: i32,
    actor_id: i64,
    value: i64,
    is_crit: bool,
    is_lucky: bool,
) -> Result<(), String> {
    let crit_hit = if is_crit { 1_i64 } else { 0_i64 };
    let lucky_hit = if is_lucky { 1_i64 } else { 0_i64 };
    diesel::sql_query(
                                    "INSERT INTO actor_encounter_stats (
                                             encounter_id, actor_id, name, class_id, ability_score, level, is_player,
                                             class_spec, is_local_player, attributes,
                                             damage_taken, hits_taken, crit_hits_taken, lucky_hits_taken, crit_total_taken, lucky_total_taken
                                     ) VALUES (
                                             ?1, ?2,
                                             (SELECT name FROM entities WHERE entity_id = ?2),
                                             NULLIF((SELECT class_id FROM entities WHERE entity_id = ?2), 0),
                                             NULLIF((SELECT ability_score FROM entities WHERE entity_id = ?2), 0),
                                             NULLIF((SELECT level FROM entities WHERE entity_id = ?2), 0),
                                             CASE WHEN EXISTS(SELECT 1 FROM entities WHERE entity_id = ?2) THEN 1 ELSE 0 END,
                                             NULLIF((SELECT class_spec FROM entities WHERE entity_id = ?2), 0),
                                             CASE WHEN ?2 = (SELECT local_player_id FROM encounters WHERE id = ?1) THEN 1 ELSE 0 END,
                                             (SELECT attributes FROM entities WHERE entity_id = ?2),
                                             ?3, 1, ?4, ?5, ?6, ?7
                                     ) ON CONFLICT(encounter_id, actor_id) DO UPDATE SET
                                        name = COALESCE(NULLIF(actor_encounter_stats.name, ''), (SELECT name FROM entities WHERE entity_id = excluded.actor_id)),
                                            class_id = COALESCE(NULLIF(actor_encounter_stats.class_id, 0), NULLIF((SELECT class_id FROM entities WHERE entity_id = excluded.actor_id), 0)),
                                            class_spec = COALESCE(NULLIF(actor_encounter_stats.class_spec, 0), NULLIF((SELECT class_spec FROM entities WHERE entity_id = excluded.actor_id), 0)),
                                        ability_score = COALESCE(NULLIF(actor_encounter_stats.ability_score, 0), NULLIF((SELECT ability_score FROM entities WHERE entity_id = excluded.actor_id), 0)),
                                        level = COALESCE(NULLIF(actor_encounter_stats.level, 0), NULLIF((SELECT level FROM entities WHERE entity_id = excluded.actor_id), 0)),
                                        attributes = COALESCE(NULLIF(actor_encounter_stats.attributes, ''), (SELECT attributes FROM entities WHERE entity_id = excluded.actor_id)),
                                         damage_taken = damage_taken + excluded.damage_taken,
                                         hits_taken = hits_taken + excluded.hits_taken,
                                         crit_hits_taken = crit_hits_taken + excluded.crit_hits_taken,
                                         lucky_hits_taken = lucky_hits_taken + excluded.lucky_hits_taken,
                                         crit_total_taken = crit_total_taken + excluded.crit_total_taken,
                                         lucky_total_taken = lucky_total_taken + excluded.lucky_total_taken"
                )
    .bind::<diesel::sql_types::Integer, _>(encounter_id)
    .bind::<diesel::sql_types::BigInt, _>(actor_id)
    .bind::<diesel::sql_types::BigInt, _>(value)
    .bind::<diesel::sql_types::BigInt, _>(crit_hit)
    .bind::<diesel::sql_types::BigInt, _>(lucky_hit)
    .bind::<diesel::sql_types::BigInt, _>(if is_crit { value } else { 0 })
    .bind::<diesel::sql_types::BigInt, _>(if is_lucky { value } else { 0 })
    .execute(conn)
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Materializes damage skill stats for an encounter.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
/// * `encounter_id` - The ID of the encounter.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result indicating success or failure.
fn materialize_damage_skill_stats(
    conn: &mut SqliteConnection,
    encounter_id: i32,
) -> Result<(), String> {
    // No-op: raw per-event tables have been removed and aggregated stats are maintained
    // incrementally at insert time. Materialization from raw events is not required.
    let _ = (conn, encounter_id);
    Ok(())
}

/// Materializes encounter bosses for an encounter.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
/// * `encounter_id` - The ID of the encounter.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result indicating success or failure.
fn materialize_encounter_bosses(
    conn: &mut SqliteConnection,
    encounter_id: i32,
) -> Result<(), String> {
    // No-op: encounter_bosses is maintained incrementally on damage inserts.
    let _ = (conn, encounter_id);
    Ok(())
}

/// Materializes heal skill stats for an encounter.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
/// * `encounter_id` - The ID of the encounter.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result indicating success or failure.
fn materialize_heal_skill_stats(
    conn: &mut SqliteConnection,
    encounter_id: i32,
) -> Result<(), String> {
    // No-op: heal_skill_stats is maintained incrementally on heal inserts.
    let _ = (conn, encounter_id);
    Ok(())
}

/// Prunes raw encounter events to save space.
///
/// # Arguments
///
/// * `conn` - A mutable reference to a `SqliteConnection`.
/// * `encounter_id` - The ID of the encounter.
///
/// # Returns
///
/// * `Result<(), String>` - An empty result indicating success or failure.
fn prune_encounter_events(conn: &mut SqliteConnection, encounter_id: i32) -> Result<(), String> {
    // No-op: no raw events table exists to prune.
    let _ = (conn, encounter_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::{DamageSkillStatRow, EncounterBossRow, HealSkillStatRow};
    // diesel prelude not needed in this scope; use fully-qualified paths where required

    fn setup_conn() -> SqliteConnection {
        use diesel::sqlite::SqliteConnection;
        let mut conn = SqliteConnection::establish(":memory:").expect("in-memory sqlite");
        // Apply migrations
        let _ = run_migrations(&mut conn).expect("migrations");
        // Pragmas similar to production
        diesel::sql_query("PRAGMA foreign_keys=ON;")
            .execute(&mut conn)
            .ok();
        conn
    }

    #[test]
    fn test_damage_taken_materialized() {
        let mut conn = setup_conn();

        // Begin encounter
        let mut enc_opt = None;
        let mut enc_start_opt = None;
        handle_task(
            &mut conn,
            DbTask::BeginEncounter {
                started_at_ms: 1000,
                local_player_id: Some(1),
                scene_id: None,
                scene_name: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();
        let enc_id = enc_opt.expect("encounter started");

        // Note: Monsters are no longer stored in the database, only players

        // Upsert defender (player)
        handle_task(
            &mut conn,
            DbTask::UpsertEntity {
                entity_id: 100,
                name: Some("Hero".into()),
                class_id: Some(1),
                class_spec: Some(1),
                ability_score: Some(1000),
                level: Some(10),
                seen_at_ms: 1000,
                attributes: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        // Insert damage event: attacker 200 -> defender 100, value 150, hp_loss 100, shield_loss 50
        handle_task(
            &mut conn,
            DbTask::InsertDamageEvent {
                timestamp_ms: 1100,
                attacker_id: 200,
                defender_id: Some(100),
                monster_name: None,
                skill_id: Some(123),
                value: 150,
                is_crit: true,
                is_lucky: false,
                hp_loss: 100,
                shield_loss: 50,
                defender_max_hp: None,
                is_boss: false,
                attempt_index: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        // Verify stats
        use sch::actor_encounter_stats::dsl as s;
        let dmg_dealt: i64 = s::actor_encounter_stats
            .select(s::damage_dealt)
            .filter(s::encounter_id.eq(enc_id).and(s::actor_id.eq(200_i64)))
            .first::<i64>(&mut conn)
            .unwrap();
        assert_eq!(dmg_dealt, 150);

        let (dmg_taken, crit_hits_taken): (i64, i64) = s::actor_encounter_stats
            .select((s::damage_taken, s::crit_hits_taken))
            .filter(s::encounter_id.eq(enc_id).and(s::actor_id.eq(100_i64)))
            .first::<(i64, i64)>(&mut conn)
            .unwrap();
        assert_eq!(dmg_taken, 150);
        assert_eq!(crit_hits_taken, 1);

        // Insert a second damage event with a monster defender and a defender name (this is the games logic for when you attack something that is a monster)
        handle_task(
            &mut conn,
            DbTask::InsertDamageEvent {
                timestamp_ms: 1150,
                attacker_id: 201,
                defender_id: Some(9999),
                monster_name: Some("Test Monster".into()),
                skill_id: Some(321),
                value: 50,
                is_crit: false,
                is_lucky: false,
                hp_loss: 50,
                shield_loss: 0,
                defender_max_hp: None,
                is_boss: true,
                attempt_index: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        // Verify monster_name persisted to encounter_bosses (we no longer store raw events)
        use sch::encounter_bosses::dsl as eb;
        let boss_rows: Vec<m::EncounterBossRow> = eb::encounter_bosses
            .filter(eb::encounter_id.eq(enc_id))
            .load::<m::EncounterBossRow>(&mut conn)
            .unwrap();
        let found = boss_rows.iter().any(|b| b.monster_name == "Test Monster");
        assert!(found, "encounter_bosses should contain Test Monster");
    }

    #[test]
    fn test_actor_snapshot_updates_identity_fields() {
        let mut conn = setup_conn();

        let mut enc_opt = None;
        let mut enc_start_opt = None;
        handle_task(
            &mut conn,
            DbTask::BeginEncounter {
                started_at_ms: 4_200,
                local_player_id: Some(10),
                scene_id: None,
                scene_name: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();
        let encounter_id = enc_opt.expect("encounter started");

        // Insert entity with unknown identity information (zeros) which should be normalized to NULL.
        handle_task(
            &mut conn,
            DbTask::UpsertEntity {
                entity_id: 10,
                name: Some("Mystery Player".into()),
                class_id: Some(0),
                class_spec: Some(0),
                ability_score: Some(0),
                level: Some(0),
                seen_at_ms: 4_250,
                attributes: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        // Trigger stats materialization.
        handle_task(
            &mut conn,
            DbTask::InsertDamageEvent {
                timestamp_ms: 4_300,
                attacker_id: 10,
                defender_id: None,
                monster_name: None,
                skill_id: Some(1001),
                value: 250,
                is_crit: false,
                is_lucky: false,
                hp_loss: 250,
                shield_loss: 0,
                defender_max_hp: None,
                is_boss: false,
                attempt_index: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        use sch::actor_encounter_stats::dsl as s;
        let initial_identity: (Option<i32>, Option<i32>, Option<i32>, Option<i32>) =
            s::actor_encounter_stats
                .select((s::class_id, s::class_spec, s::ability_score, s::level))
                .filter(s::encounter_id.eq(encounter_id).and(s::actor_id.eq(10_i64)))
                .first(&mut conn)
                .unwrap();
        assert_eq!(initial_identity, (None, None, None, None));

        // Update entity with real identity values (which should now backfill the snapshot row).
        handle_task(
            &mut conn,
            DbTask::UpsertEntity {
                entity_id: 10,
                name: Some("Mystery Player".into()),
                class_id: Some(2),
                class_spec: Some(7),
                ability_score: Some(1_234),
                level: Some(35),
                seen_at_ms: 4_400,
                attributes: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        // A subsequent event should refresh the snapshot immediately.
        handle_task(
            &mut conn,
            DbTask::InsertDamageEvent {
                timestamp_ms: 4_450,
                attacker_id: 10,
                defender_id: None,
                monster_name: None,
                skill_id: Some(1002),
                value: 150,
                is_crit: true,
                is_lucky: false,
                hp_loss: 150,
                shield_loss: 0,
                defender_max_hp: None,
                is_boss: false,
                attempt_index: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        let refreshed_identity: (Option<i32>, Option<i32>, Option<i32>, Option<i32>) =
            s::actor_encounter_stats
                .select((s::class_id, s::class_spec, s::ability_score, s::level))
                .filter(s::encounter_id.eq(encounter_id).and(s::actor_id.eq(10_i64)))
                .first(&mut conn)
                .unwrap();
        assert_eq!(
            refreshed_identity,
            (Some(2), Some(7), Some(1_234), Some(35))
        );

        // Ending the encounter should also work with fully populated data and leave identity intact.
        handle_task(
            &mut conn,
            DbTask::EndEncounter {
                ended_at_ms: 4_600,
                defeated_bosses: None,
                is_manually_reset: false,
                player_active_times: Vec::new(),
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        // Ensure the encounter id has been cleared after ending.
        assert!(enc_opt.is_none());
    }

    #[test]
    fn test_end_all_active_encounters_finishes_dangling_rows() {
        use sch::encounters::dsl as e;

        let mut conn = setup_conn();

        let mut enc_opt = None;
        let mut enc_start_opt = None;
        handle_task(
            &mut conn,
            DbTask::BeginEncounter {
                started_at_ms: 1_000,
                local_player_id: Some(42),
                scene_id: None,
                scene_name: Some("Test Scene".into()),
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        // Ensure some combat data exists so finalize logic runs typical rollups
        handle_task(
            &mut conn,
            DbTask::InsertDamageEvent {
                timestamp_ms: 1_100,
                attacker_id: 42,
                defender_id: Some(7),
                monster_name: Some("Training Dummy".into()),
                skill_id: Some(10),
                value: 50,
                is_crit: false,
                is_lucky: false,
                hp_loss: 50,
                shield_loss: 0,
                defender_max_hp: None,
                is_boss: false,
                attempt_index: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        // Simulate startup cleanup
        handle_task(
            &mut conn,
            DbTask::EndAllActiveEncounters { ended_at_ms: 2_000 },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        // Current encounter pointer should be cleared
        assert!(enc_opt.is_none());

        // Encounter should be marked ended with duration populated
        let (ended_at_ms, duration): (Option<i64>, f64) = e::encounters
            .select((e::ended_at_ms, e::duration))
            .first(&mut conn)
            .unwrap();
        assert_eq!(ended_at_ms, Some(2_000));
        assert!(duration >= 1.0);
    }

    #[test]
    fn test_rollups_materialized_and_raw_pruned() {
        let mut conn = setup_conn();

        let mut enc_opt = None;
        let mut enc_start_opt = None;
        handle_task(
            &mut conn,
            DbTask::BeginEncounter {
                started_at_ms: 10_000,
                local_player_id: Some(1),
                scene_id: None,
                scene_name: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();
        let encounter_id = enc_opt.expect("encounter started");

        handle_task(
            &mut conn,
            DbTask::InsertDamageEvent {
                timestamp_ms: 10_050,
                attacker_id: 200,
                defender_id: Some(100),
                monster_name: Some("Ancient Construct".into()),
                skill_id: Some(321),
                value: 500,
                is_crit: true,
                is_lucky: false,
                hp_loss: 500,
                shield_loss: 0,
                defender_max_hp: None,
                is_boss: true,
                attempt_index: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        handle_task(
            &mut conn,
            DbTask::InsertDamageEvent {
                timestamp_ms: 10_060,
                attacker_id: 200,
                defender_id: Some(101),
                monster_name: None,
                skill_id: None,
                value: 150,
                is_crit: false,
                is_lucky: true,
                hp_loss: 100,
                shield_loss: 50,
                defender_max_hp: None,
                is_boss: false,
                attempt_index: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        handle_task(
            &mut conn,
            DbTask::InsertHealEvent {
                timestamp_ms: 10_070,
                healer_id: 300,
                target_id: Some(200),
                skill_id: Some(777),
                value: 420,
                is_crit: true,
                is_lucky: false,
                attempt_index: None,
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();

        handle_task(
            &mut conn,
            DbTask::EndEncounter {
                ended_at_ms: 10_500,
                defeated_bosses: None,
                is_manually_reset: false,
                player_active_times: Vec::new(),
            },
            &mut enc_opt,
            &mut enc_start_opt,
        )
        .unwrap();
        assert!(enc_opt.is_none());

        use sch::damage_skill_stats::dsl as dss;
        use sch::encounter_bosses::dsl as eb;
        use sch::heal_skill_stats::dsl as hss;

        let damage_rows = dss::damage_skill_stats
            .filter(dss::encounter_id.eq(encounter_id))
            .order((dss::attacker_id, dss::skill_id))
            .load::<DamageSkillStatRow>(&mut conn)
            .unwrap();
        assert_eq!(damage_rows.len(), 2);
        let explicit = damage_rows
            .iter()
            .find(|row| row.skill_id == 321)
            .expect("missing explicit skill aggregate");
        assert_eq!(explicit.hits, 1);
        assert_eq!(explicit.total_value, 500);
        assert_eq!(explicit.crit_hits, 1);

        let unknown = damage_rows
            .iter()
            .find(|row| row.skill_id == super::UNKNOWN_SKILL_ID_SENTINEL)
            .expect("missing unknown skill aggregate");
        assert_eq!(unknown.total_value, 150);
        assert_eq!(unknown.lucky_hits, 1);

        let heal_rows = hss::heal_skill_stats
            .filter(hss::encounter_id.eq(encounter_id))
            .load::<HealSkillStatRow>(&mut conn)
            .unwrap();
        assert_eq!(heal_rows.len(), 1);
        assert_eq!(heal_rows[0].skill_id, 777);
        assert_eq!(heal_rows[0].total_value, 420);

        let boss_rows = eb::encounter_bosses
            .filter(eb::encounter_id.eq(encounter_id))
            .load::<EncounterBossRow>(&mut conn)
            .unwrap();
        assert_eq!(boss_rows.len(), 1);
        assert_eq!(boss_rows[0].monster_name, "Ancient Construct");
        assert_eq!(boss_rows[0].hits, 1);

        // Raw event tables no longer exist; aggregates are created incrementally.
    }
}
