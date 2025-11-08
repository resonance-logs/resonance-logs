PRAGMA foreign_keys = OFF;

-- Drop indexes related to raw event tables (if present)
DROP INDEX IF EXISTS idx_damage_events_encounter;
DROP INDEX IF EXISTS idx_damage_events_attacker_time;
DROP INDEX IF EXISTS idx_damage_events_defender_time;
DROP INDEX IF EXISTS idx_damage_events_skill;
DROP INDEX IF EXISTS idx_damage_events_boss;
DROP INDEX IF EXISTS idx_damage_events_encounter_boss;
DROP INDEX IF EXISTS idx_damage_events_monster_name;
DROP INDEX IF EXISTS idx_damage_events_attempt;

DROP INDEX IF EXISTS idx_heal_events_encounter;
DROP INDEX IF EXISTS idx_heal_events_healer_time;
DROP INDEX IF EXISTS idx_heal_events_target_time;
DROP INDEX IF EXISTS idx_heal_events_skill;
DROP INDEX IF EXISTS idx_heal_events_attempt;

-- Drop the raw per-event tables
DROP TABLE IF EXISTS damage_events;
DROP TABLE IF EXISTS heal_events;

PRAGMA foreign_keys = ON;
