pub mod commands;
pub mod models;
pub mod schema;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::database::models as m;
use crate::database::schema as sch;

#[allow(dead_code)]
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();

static DB_SENDER: Lazy<Mutex<Option<UnboundedSender<DbTask>>>> = Lazy::new(|| Mutex::new(None));

// Batch configuration: collect up to this many events or wait up to this many ms (whichever
// comes first) before flushing to the DB. Chosen by user: 50ms window.
const BATCH_MAX_EVENTS: usize = 100;
const BATCH_MAX_WAIT_MS: u64 = 50;

#[derive(Debug, thiserror::Error)]
pub enum DbInitError {
    #[error("DB pool error: {0}")]
    Pool(String),
    #[error("Migration error: {0}")]
    Migration(String),
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

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

pub fn ensure_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

pub fn init_and_spawn_writer() -> Result<(), DbInitError> {
    let db_path = default_db_path();
    if let Err(e) = ensure_parent_dir(&db_path) {
        return Err(DbInitError::Pool(format!("failed to create dir: {e}")));
    }

    let manager = ConnectionManager::<SqliteConnection>::new(db_path.to_string_lossy().to_string());
    // Increase connection pool size to reduce contention for DB connections
    // under concurrent UI reads + writer work. Tuned to 8 as a reasonable default (probably)
    let pool = Pool::builder()
        .max_size(8)
        .build(manager)
        .map_err(|e| DbInitError::Pool(e.to_string()))?;

    // Run migrations once
    {
        let mut conn = pool.get().map_err(|e| DbInitError::Pool(e.to_string()))?;
        run_migrations(&mut conn).map_err(|e| DbInitError::Migration(e))?;
        // Pragmas for better concurrency/perf
        diesel::sql_query("PRAGMA journal_mode=WAL;")
            .execute(&mut conn)
            .ok();
        diesel::sql_query("PRAGMA synchronous=NORMAL;")
            .execute(&mut conn)
            .ok();
        diesel::sql_query("PRAGMA foreign_keys=ON;")
            .execute(&mut conn)
            .ok();
    }

    // Spawn writer worker using an unbounded channel (grow-on-demand) and batch
    // incoming tasks for up to BATCH_MAX_WAIT_MS or until BATCH_MAX_EVENTS is
    // reached. Using an unbounded channel avoids silent drops on overflow and
    // satisfies the requirement to "increase queue size" when under pressure.
    let (tx, mut rx) = mpsc::unbounded_channel::<DbTask>();
    {
        let mut guard = DB_SENDER.lock();
        *guard = Some(tx.clone());
    }

    std::thread::spawn(move || {
        let mut current_encounter_id: Option<i32> = None;
        loop {
            // Block until we receive the first task
            let first = rx.blocking_recv();
            let Some(first) = first else {
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
                    log::error!("DB get conn: {e}");
                    continue;
                }
            };

            let batch_result = conn.transaction::<(), diesel::result::Error, _>(|tx_conn| {
                for task in tasks.iter().cloned() {
                    handle_task(tx_conn, task, &mut current_encounter_id).map_err(|e| {
                        log::error!("DB task in batch failed: {}", e);
                        diesel::result::Error::RollbackTransaction
                    })?;
                }
                Ok(())
            });

            if let Err(e) = batch_result {
                log::error!(
                    "Batch transaction failed: {:?}. Falling back to per-task execution.",
                    e
                );

                // CRITICAL: Reset current_encounter_id because the batch transaction was rolled back.
                // Any encounter that was created in the failed transaction no longer exists.
                // This ensures BeginEncounter tasks will create new encounters instead of being skipped.
                current_encounter_id = None;

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
                    if let Err(e) = handle_task(&mut conn, task, &mut current_encounter_id) {
                        log::error!("DB task error (fallback - BeginEncounter): {}", e);
                    }
                }

                // Then process all other tasks
                for task in other_tasks {
                    if let Err(e) = handle_task(&mut conn, task, &mut current_encounter_id) {
                        log::error!("DB task error (fallback): {}", e);
                    }
                }
            }
        }
    });

    Ok(())
}

fn run_migrations(conn: &mut SqliteConnection) -> Result<(), String> {
    conn.run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
pub enum DbTask {
    BeginEncounter {
        started_at_ms: i64,
        local_player_id: Option<i64>,
    },
    EndEncounter {
        ended_at_ms: i64,
    },

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

    UpsertSkill {
        skill_id: i32,
        name: Option<String>,
    },

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
        is_boss: bool,
    },

    InsertHealEvent {
        timestamp_ms: i64,
        healer_id: i64,
        target_id: Option<i64>,
        skill_id: Option<i32>,
        value: i64,
        is_crit: bool,
        is_lucky: bool,
    },
}

pub fn enqueue(task: DbTask) {
    // fire-and-forget; drop if not initialized
    let guard = DB_SENDER.lock();
    if let Some(tx) = guard.as_ref() {
        if let Err(e) = tx.send(task) {
            // Unbounded sender should not overflow; an error means the receiver was dropped.
            log::warn!("DB queue send failed (receiver closed): {}", e);
        }
    }
}

fn handle_task(
    conn: &mut SqliteConnection,
    task: DbTask,
    current_encounter_id: &mut Option<i32>,
) -> Result<(), String> {
    match task {
        DbTask::BeginEncounter {
            started_at_ms,
            local_player_id,
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
        }
        DbTask::EndEncounter { ended_at_ms } => {
            if let Some(id) = current_encounter_id.take() {
                use sch::encounters::dsl as e;
                diesel::update(e::encounters.filter(e::id.eq(id)))
                    .set(e::ended_at_ms.eq(ended_at_ms))
                    .execute(conn)
                    .map_err(|er| er.to_string())?;
            }
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
            use sch::entities::dsl as en;
            let exists: Option<i64> = en::entities
                .select(en::entity_id)
                .filter(en::entity_id.eq(entity_id))
                .first::<i64>(conn)
                .optional()
                .map_err(|e| e.to_string())?;
            if exists.is_some() {
                let update = m::UpdateEntity {
                    name: name.as_deref(),
                    class_id,
                    class_spec,
                    ability_score,
                    level,
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
                    name: name.as_deref(),
                    class_id,
                    class_spec,
                    ability_score,
                    level,
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
        DbTask::UpsertSkill { skill_id, name } => {
            use sch::skills::dsl as sk;
            let exists: Option<i32> = sk::skills
                .select(sk::skill_id)
                .filter(sk::skill_id.eq(skill_id))
                .first::<i32>(conn)
                .optional()
                .map_err(|e| e.to_string())?;
            if exists.is_some() {
                // Update name only if provided
                if let Some(n) = name.as_deref() {
                    diesel::update(sk::skills.filter(sk::skill_id.eq(skill_id)))
                        .set(sk::name.eq(n))
                        .execute(conn)
                        .map_err(|e| e.to_string())?;
                }
            } else {
                let insert = m::NewSkill {
                    skill_id,
                    name: name.as_deref(),
                };
                diesel::insert_into(sk::skills)
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
            is_boss,
        } => {
            if let Some(enc_id) = *current_encounter_id {
                use sch::damage_events::dsl as d;
                let ins = m::NewDamageEvent {
                    encounter_id: enc_id,
                    timestamp_ms,
                    attacker_id,
                    defender_id,
                    monster_name,
                    skill_id,
                    value,
                    is_crit: if is_crit { 1 } else { 0 },
                    is_lucky: if is_lucky { 1 } else { 0 },
                    hp_loss,
                    shield_loss,
                    is_boss: if is_boss { 1 } else { 0 },
                };
                diesel::insert_into(d::damage_events)
                    .values(&ins)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
                // increment encounter totals
                diesel::sql_query(
                    "UPDATE encounters SET total_dmg = COALESCE(total_dmg,0) + ?1 WHERE id = ?2",
                )
                .bind::<diesel::sql_types::BigInt, _>(value)
                .bind::<diesel::sql_types::Integer, _>(enc_id)
                .execute(conn)
                .ok();

                // Materialize per-actor stats: attacker damage_dealt; defender damage_taken
                // Attacker
                upsert_stats_add_damage_dealt(
                    conn,
                    enc_id,
                    attacker_id,
                    value,
                    is_crit,
                    is_lucky,
                    is_boss,
                )?;

                // Defender (damage taken). Follow live semantics: only count taken when attacker is not a player.
                if let Some(def_id) = defender_id {
                    if let Some(attacker_type) = get_entity_type(conn, attacker_id)? {
                        // EEntityType::EntChar == 1 (verify enum mapping). In our code we store entity_type as i32 from blueprotobuf.
                        if attacker_type
                            != (blueprotobuf_lib::blueprotobuf::EEntityType::EntChar as i32)
                        {
                            // Prefer hp_loss + shield_loss if provided; but 'value' has already been set to that when >0 in producer.
                            upsert_stats_add_damage_taken(
                                conn, enc_id, def_id, value, is_crit, is_lucky,
                            )?;
                        }
                    } else {
                        // If attacker type unknown, conservatively include as taken
                        upsert_stats_add_damage_taken(
                            conn, enc_id, def_id, value, is_crit, is_lucky,
                        )?;
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
        } => {
            if let Some(enc_id) = *current_encounter_id {
                use sch::heal_events::dsl as h;
                let ins = m::NewHealEvent {
                    encounter_id: enc_id,
                    timestamp_ms,
                    healer_id,
                    target_id,
                    skill_id,
                    value,
                    is_crit: if is_crit { 1 } else { 0 },
                    is_lucky: if is_lucky { 1 } else { 0 },
                };
                diesel::insert_into(h::heal_events)
                    .values(&ins)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
                // increment encounter totals
                diesel::sql_query(
                    "UPDATE encounters SET total_heal = COALESCE(total_heal,0) + ?1 WHERE id = ?2",
                )
                .bind::<diesel::sql_types::BigInt, _>(value)
                .bind::<diesel::sql_types::Integer, _>(enc_id)
                .execute(conn)
                .ok();

                // Materialize per-actor stats: healer heal_dealt
                upsert_stats_add_heal_dealt(conn, enc_id, healer_id, value, is_crit, is_lucky)?;
            }
        }
    }
    Ok(())
}

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
                        damage_dealt, hits_dealt, crit_hits_dealt, lucky_hits_dealt, crit_total_dealt, lucky_total_dealt,
                        boss_damage_dealt, boss_hits_dealt, boss_crit_hits_dealt, boss_lucky_hits_dealt, boss_crit_total_dealt, boss_lucky_total_dealt
                 ) VALUES (
                        ?1, ?2,
                        (SELECT name FROM entities WHERE entity_id = ?2),
                        (SELECT class_id FROM entities WHERE entity_id = ?2),
                        (SELECT ability_score FROM entities WHERE entity_id = ?2),
                        (SELECT level FROM entities WHERE entity_id = ?2),
                        CASE WHEN EXISTS(SELECT 1 FROM entities WHERE entity_id = ?2) THEN 1 ELSE 0 END,
                        ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13
                 ) ON CONFLICT(encounter_id, actor_id) DO UPDATE SET
                     name = COALESCE(actor_encounter_stats.name, (SELECT name FROM entities WHERE entity_id = excluded.actor_id)),
                     class_id = COALESCE(actor_encounter_stats.class_id, (SELECT class_id FROM entities WHERE entity_id = excluded.actor_id)),
                     ability_score = COALESCE(actor_encounter_stats.ability_score, (SELECT ability_score FROM entities WHERE entity_id = excluded.actor_id)),
                     level = COALESCE(actor_encounter_stats.level, (SELECT level FROM entities WHERE entity_id = excluded.actor_id)),
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
                     boss_lucky_total_dealt = boss_lucky_total_dealt + excluded.boss_lucky_total_dealt"
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
    .execute(conn)
    .map(|_| ())
    .map_err(|e| e.to_string())
}

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
                        heal_dealt, hits_heal, crit_hits_heal, lucky_hits_heal, crit_total_heal, lucky_total_heal
                 ) VALUES (
                        ?1, ?2,
                        (SELECT name FROM entities WHERE entity_id = ?2),
                        (SELECT class_id FROM entities WHERE entity_id = ?2),
                        (SELECT ability_score FROM entities WHERE entity_id = ?2),
                        (SELECT level FROM entities WHERE entity_id = ?2),
                        CASE WHEN EXISTS(SELECT 1 FROM entities WHERE entity_id = ?2) THEN 1 ELSE 0 END,
                        ?3, 1, ?4, ?5, ?6, ?7
                 ) ON CONFLICT(encounter_id, actor_id) DO UPDATE SET
                     name = COALESCE(actor_encounter_stats.name, (SELECT name FROM entities WHERE entity_id = excluded.actor_id)),
                     class_id = COALESCE(actor_encounter_stats.class_id, (SELECT class_id FROM entities WHERE entity_id = excluded.actor_id)),
                     ability_score = COALESCE(actor_encounter_stats.ability_score, (SELECT ability_score FROM entities WHERE entity_id = excluded.actor_id)),
                     level = COALESCE(actor_encounter_stats.level, (SELECT level FROM entities WHERE entity_id = excluded.actor_id)),
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
                        damage_taken, hits_taken, crit_hits_taken, lucky_hits_taken, crit_total_taken, lucky_total_taken
                 ) VALUES (
                        ?1, ?2,
                        (SELECT name FROM entities WHERE entity_id = ?2),
                        (SELECT class_id FROM entities WHERE entity_id = ?2),
                        (SELECT ability_score FROM entities WHERE entity_id = ?2),
                        (SELECT level FROM entities WHERE entity_id = ?2),
                        CASE WHEN EXISTS(SELECT 1 FROM entities WHERE entity_id = ?2) THEN 1 ELSE 0 END,
                        ?3, 1, ?4, ?5, ?6, ?7
                 ) ON CONFLICT(encounter_id, actor_id) DO UPDATE SET
                     name = COALESCE(actor_encounter_stats.name, (SELECT name FROM entities WHERE entity_id = excluded.actor_id)),
                     class_id = COALESCE(actor_encounter_stats.class_id, (SELECT class_id FROM entities WHERE entity_id = excluded.actor_id)),
                     ability_score = COALESCE(actor_encounter_stats.ability_score, (SELECT ability_score FROM entities WHERE entity_id = excluded.actor_id)),
                     level = COALESCE(actor_encounter_stats.level, (SELECT level FROM entities WHERE entity_id = excluded.actor_id)),
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

#[cfg(test)]
mod tests {
    use super::*;
    // diesel prelude not needed in this scope; use fully-qualified paths where required

    fn setup_conn() -> SqliteConnection {
        use diesel::sqlite::SqliteConnection;
        let mut conn = SqliteConnection::establish(":memory:").expect("in-memory sqlite");
        // Apply migrations
        run_migrations(&mut conn).expect("migrations");
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
        handle_task(
            &mut conn,
            DbTask::BeginEncounter {
                started_at_ms: 1000,
                local_player_id: Some(1),
            },
            &mut enc_opt,
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
                is_boss: false,
            },
            &mut enc_opt,
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
                is_boss: true,
            },
            &mut enc_opt,
        )
        .unwrap();

        // Verify monster_name persisted
        #[derive(diesel::QueryableByName)]
        struct NameRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            monster_name: Option<String>,
        }
        let row: NameRow = diesel::sql_query(
            "SELECT monster_name FROM damage_events WHERE attacker_id = ?1 AND defender_id = ?2",
        )
        .bind::<diesel::sql_types::BigInt, _>(201_i64)
        .bind::<diesel::sql_types::BigInt, _>(9999_i64)
        .get_result::<NameRow>(&mut conn)
        .unwrap();
        assert_eq!(row.monster_name.as_deref(), Some("Test Monster"));
    }
}
