PRAGMA foreign_keys = ON;

DROP INDEX IF EXISTS idx_death_events_actor_time;
DROP INDEX IF EXISTS idx_death_events_encounter;
DROP TABLE IF EXISTS death_events;
