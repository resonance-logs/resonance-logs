pub mod models;
pub mod schema;
pub mod commands;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use once_cell::sync::Lazy;
use tokio::sync::mpsc::{self, Sender};
use parking_lot::Mutex;

use crate::database::models as m;
use crate::database::schema as sch;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!();

static DB_SENDER: Lazy<Mutex<Option<Sender<DbTask>>>> = Lazy::new(|| Mutex::new(None));

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
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("resonance-logs.db")
    }
}

pub fn ensure_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(dir) = path.parent() { std::fs::create_dir_all(dir)?; }
    Ok(())
}

pub fn init_and_spawn_writer() -> Result<(), DbInitError> {
    let db_path = default_db_path();
    if let Err(e) = ensure_parent_dir(&db_path) { return Err(DbInitError::Pool(format!("failed to create dir: {e}"))); }

    let manager = ConnectionManager::<SqliteConnection>::new(db_path.to_string_lossy().to_string());
    let pool = Pool::builder()
        .max_size(4)
        .build(manager)
        .map_err(|e| DbInitError::Pool(e.to_string()))?;

    // Run migrations once
    {
        let mut conn = pool.get().map_err(|e| DbInitError::Pool(e.to_string()))?;
        run_migrations(&mut conn).map_err(|e| DbInitError::Migration(e))?;
        // Pragmas for better concurrency/perf
        diesel::sql_query("PRAGMA journal_mode=WAL;").execute(&mut conn).ok();
        diesel::sql_query("PRAGMA synchronous=NORMAL;").execute(&mut conn).ok();
        diesel::sql_query("PRAGMA foreign_keys=ON;").execute(&mut conn).ok();
    }

    // Spawn writer worker
    let (tx, mut rx) = mpsc::channel::<DbTask>(10_000);
    {
        let mut guard = DB_SENDER.lock();
        *guard = Some(tx.clone());
    }

    std::thread::spawn(move || {
        // blocking writer thread
        let mut current_encounter_id: Option<i32> = None;
        loop {
            let task = rx.blocking_recv();
            let Some(task) = task else { break; };
            let mut conn = match pool.get() { Ok(c) => c, Err(e) => { log::error!("DB get conn: {e}"); continue; } };
            match handle_task(&mut conn, task, &mut current_encounter_id) {
                Ok(_) => {}
                Err(e) => log::error!("DB task error: {e}")
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

#[derive(Debug)]
pub enum DbTask {
    BeginEncounter { started_at_ms: i64, local_player_id: Option<i64> },
    EndEncounter { ended_at_ms: i64 },

    UpsertEntity {
        entity_id: i64,
        entity_type: i32,
        is_player: bool,
        name: Option<String>,
        class_id: Option<i32>,
        class_spec: Option<i32>,
        ability_score: Option<i32>,
        level: Option<i32>,
        seen_at_ms: i64,
    },

    UpsertSkill { skill_id: i32, name: Option<String> },

    InsertDamageEvent {
        timestamp_ms: i64,
        attacker_id: i64,
        defender_id: Option<i64>,
        skill_id: Option<i32>,
        value: i64,
        is_crit: bool,
        is_lucky: bool,
        hp_loss: i64,
        shield_loss: i64,
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
        if let Err(e) = tx.try_send(task) {
            log::warn!("DB queue full or closed, dropping task: {}", e);
        }
    }
}

fn handle_task(conn: &mut SqliteConnection, task: DbTask, current_encounter_id: &mut Option<i32>) -> Result<(), String> {
    match task {
        DbTask::BeginEncounter { started_at_ms, local_player_id } => {
            if current_encounter_id.is_some() { return Ok(()); }
            use sch::encounters::dsl as e;
            let new_enc = m::NewEncounter {
                session_id: None,
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
            let id: i32 = e::encounters.order(e::id.desc()).select(e::id).first::<i32>(conn).map_err(|e| e.to_string())?;
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
        DbTask::UpsertEntity { entity_id, entity_type, is_player, name, class_id, class_spec, ability_score, level, seen_at_ms } => {
            use sch::entities::dsl as en;
            let exists: Option<i64> = en::entities
                .select(en::entity_id)
                .filter(en::entity_id.eq(entity_id))
                .first::<i64>(conn)
                .optional()
                .map_err(|e| e.to_string())?;
            if exists.is_some() {
                let update = m::UpdateEntity {
                    entity_type: Some(entity_type),
                    is_player: Some(if is_player {1} else {0}),
                    name: name.as_deref(),
                    class_id,
                    class_spec,
                    ability_score,
                    level,
                    last_seen_ms: Some(seen_at_ms),
                };
                diesel::update(en::entities.filter(en::entity_id.eq(entity_id)))
                    .set(&update)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
            } else {
                let insert = m::NewEntity {
                    entity_id,
                    entity_type,
                    is_player: if is_player {1} else {0},
                    name: name.as_deref(),
                    class_id,
                    class_spec,
                    ability_score,
                    level,
                    first_seen_ms: Some(seen_at_ms),
                    last_seen_ms: Some(seen_at_ms),
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
                let insert = m::NewSkill { skill_id, name: name.as_deref() };
                diesel::insert_into(sk::skills)
                    .values(&insert)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
            }
        }
        DbTask::InsertDamageEvent { timestamp_ms, attacker_id, defender_id, skill_id, value, is_crit, is_lucky, hp_loss, shield_loss } => {
            if let Some(enc_id) = *current_encounter_id {
                use sch::damage_events::dsl as d;
                let ins = m::NewDamageEvent {
                    encounter_id: enc_id,
                    timestamp_ms,
                    attacker_id,
                    defender_id,
                    skill_id,
                    value,
                    is_crit: if is_crit {1} else {0},
                    is_lucky: if is_lucky {1} else {0},
                    hp_loss,
                    shield_loss,
                };
                diesel::insert_into(d::damage_events)
                    .values(&ins)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
                // increment encounter totals
                diesel::sql_query("UPDATE encounters SET total_dmg = COALESCE(total_dmg,0) + ?1 WHERE id = ?2")
                    .bind::<diesel::sql_types::BigInt, _>(value)
                    .bind::<diesel::sql_types::Integer, _>(enc_id)
                    .execute(conn)
                    .ok();

                // Materialize per-actor stats: attacker damage_dealt; defender damage_taken
                // Attacker
                upsert_stats_add_damage_dealt(conn, enc_id, attacker_id, value, is_crit, is_lucky)?;

                // Defender (damage taken). Follow live semantics: only count taken when attacker is not a player.
                if let Some(def_id) = defender_id {
                    if let Some(attacker_type) = get_entity_type(conn, attacker_id)? {
                        // EEntityType::EntChar == 1 (verify enum mapping). In our code we store entity_type as i32 from blueprotobuf.
                        if attacker_type != (blueprotobuf_lib::blueprotobuf::EEntityType::EntChar as i32) {
                            // Prefer hp_loss + shield_loss if provided; but 'value' has already been set to that when >0 in producer.
                            upsert_stats_add_damage_taken(conn, enc_id, def_id, value, is_crit, is_lucky)?;
                        }
                    } else {
                        // If attacker type unknown, conservatively include as taken
                        upsert_stats_add_damage_taken(conn, enc_id, def_id, value, is_crit, is_lucky)?;
                    }
                }
            }
        }
        DbTask::InsertHealEvent { timestamp_ms, healer_id, target_id, skill_id, value, is_crit, is_lucky } => {
            if let Some(enc_id) = *current_encounter_id {
                use sch::heal_events::dsl as h;
                let ins = m::NewHealEvent {
                    encounter_id: enc_id,
                    timestamp_ms,
                    healer_id,
                    target_id,
                    skill_id,
                    value,
                    is_crit: if is_crit {1} else {0},
                    is_lucky: if is_lucky {1} else {0},
                };
                diesel::insert_into(h::heal_events)
                    .values(&ins)
                    .execute(conn)
                    .map_err(|e| e.to_string())?;
                // increment encounter totals
                diesel::sql_query("UPDATE encounters SET total_heal = COALESCE(total_heal,0) + ?1 WHERE id = ?2")
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
    use sch::entities::dsl as en;
    en::entities
        .select(en::entity_type)
        .filter(en::entity_id.eq(entity_id))
        .first::<i32>(conn)
        .optional()
        .map_err(|e| e.to_string())
}

fn upsert_stats_add_damage_dealt(conn: &mut SqliteConnection, encounter_id: i32, actor_id: i64, value: i64, is_crit: bool, is_lucky: bool) -> Result<(), String> {
    let crit_hit = if is_crit { 1_i64 } else { 0_i64 };
    let lucky_hit = if is_lucky { 1_i64 } else { 0_i64 };
    diesel::sql_query(
        "INSERT INTO actor_encounter_stats (encounter_id, actor_id, damage_dealt, hits_dealt, crit_hits_dealt, lucky_hits_dealt, crit_total_dealt, lucky_total_dealt) \
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7) \
         ON CONFLICT(encounter_id, actor_id) DO UPDATE SET \
           damage_dealt = damage_dealt + excluded.damage_dealt, \
           hits_dealt = hits_dealt + excluded.hits_dealt, \
           crit_hits_dealt = crit_hits_dealt + excluded.crit_hits_dealt, \
           lucky_hits_dealt = lucky_hits_dealt + excluded.lucky_hits_dealt, \
           crit_total_dealt = crit_total_dealt + excluded.crit_total_dealt, \
           lucky_total_dealt = lucky_total_dealt + excluded.lucky_total_dealt"
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

fn upsert_stats_add_heal_dealt(conn: &mut SqliteConnection, encounter_id: i32, actor_id: i64, value: i64, is_crit: bool, is_lucky: bool) -> Result<(), String> {
    let crit_hit = if is_crit { 1_i64 } else { 0_i64 };
    let lucky_hit = if is_lucky { 1_i64 } else { 0_i64 };
    diesel::sql_query(
        "INSERT INTO actor_encounter_stats (encounter_id, actor_id, heal_dealt, hits_heal, crit_hits_heal, lucky_hits_heal, crit_total_heal, lucky_total_heal) \
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7) \
         ON CONFLICT(encounter_id, actor_id) DO UPDATE SET \
           heal_dealt = heal_dealt + excluded.heal_dealt, \
           hits_heal = hits_heal + excluded.hits_heal, \
           crit_hits_heal = crit_hits_heal + excluded.crit_hits_heal, \
           lucky_hits_heal = lucky_hits_heal + excluded.lucky_hits_heal, \
           crit_total_heal = crit_total_heal + excluded.crit_total_heal, \
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

fn upsert_stats_add_damage_taken(conn: &mut SqliteConnection, encounter_id: i32, actor_id: i64, value: i64, is_crit: bool, is_lucky: bool) -> Result<(), String> {
    let crit_hit = if is_crit { 1_i64 } else { 0_i64 };
    let lucky_hit = if is_lucky { 1_i64 } else { 0_i64 };
    diesel::sql_query(
        "INSERT INTO actor_encounter_stats (encounter_id, actor_id, damage_taken, hits_taken, crit_hits_taken, lucky_hits_taken, crit_total_taken, lucky_total_taken) \
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7) \
         ON CONFLICT(encounter_id, actor_id) DO UPDATE SET \
           damage_taken = damage_taken + excluded.damage_taken, \
           hits_taken = hits_taken + excluded.hits_taken, \
           crit_hits_taken = crit_hits_taken + excluded.crit_hits_taken, \
           lucky_hits_taken = lucky_hits_taken + excluded.lucky_hits_taken, \
           crit_total_taken = crit_total_taken + excluded.crit_total_taken, \
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
    use diesel::prelude::*;

    fn setup_conn() -> SqliteConnection {
        use diesel::sqlite::SqliteConnection;
        let mut conn = SqliteConnection::establish(":memory:").expect("in-memory sqlite");
        // Apply migrations
        run_migrations(&mut conn).expect("migrations");
        // Pragmas similar to production
        diesel::sql_query("PRAGMA foreign_keys=ON;").execute(&mut conn).ok();
        conn
    }

    #[test]
    fn test_damage_taken_materialized() {
        let mut conn = setup_conn();

        // Begin encounter
        let mut enc_opt = None;
        handle_task(&mut conn, DbTask::BeginEncounter { started_at_ms: 1000, local_player_id: Some(1) }, &mut enc_opt).unwrap();
        let enc_id = enc_opt.expect("encounter started");

        // Upsert attacker (monster)
        handle_task(&mut conn, DbTask::UpsertEntity {
            entity_id: 200,
            entity_type: blueprotobuf_lib::blueprotobuf::EEntityType::EntMonster as i32,
            is_player: false,
            name: Some("Goblin".into()),
            class_id: None,
            class_spec: None,
            ability_score: None,
            level: None,
            seen_at_ms: 1000,
        }, &mut enc_opt).unwrap();

        // Upsert defender (player)
        handle_task(&mut conn, DbTask::UpsertEntity {
            entity_id: 100,
            entity_type: blueprotobuf_lib::blueprotobuf::EEntityType::EntChar as i32,
            is_player: true,
            name: Some("Hero".into()),
            class_id: Some(1),
            class_spec: Some(1),
            ability_score: Some(1000),
            level: Some(10),
            seen_at_ms: 1000,
        }, &mut enc_opt).unwrap();

        // Insert damage event: attacker 200 -> defender 100, value 150, hp_loss 100, shield_loss 50
        handle_task(&mut conn, DbTask::InsertDamageEvent {
            timestamp_ms: 1100,
            attacker_id: 200,
            defender_id: Some(100),
            skill_id: Some(123),
            value: 150,
            is_crit: true,
            is_lucky: false,
            hp_loss: 100,
            shield_loss: 50,
        }, &mut enc_opt).unwrap();

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
    }
}
