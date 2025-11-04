-- Remove columns added in up.sql
-- Note: SQLite doesn't support DROP COLUMN prior to newer versions; this down.sql uses a conservative approach if supported. If not supported, manual migration rollback required.
PRAGMA foreign_keys=off;
BEGIN TRANSACTION;

-- Recreate damage_events without defender_max_hp
CREATE TABLE damage_events_new (
    id INTEGER PRIMARY KEY,
    encounter_id INTEGER NOT NULL,
    timestamp_ms BIGINT NOT NULL,
    attacker_id BIGINT NOT NULL,
    defender_id BIGINT,
    monster_name TEXT,
    skill_id INTEGER,
    value BIGINT NOT NULL,
    is_crit INTEGER NOT NULL,
    is_lucky INTEGER NOT NULL,
    hp_loss BIGINT NOT NULL,
    shield_loss BIGINT NOT NULL,
    is_boss INTEGER NOT NULL
);
INSERT INTO damage_events_new (id, encounter_id, timestamp_ms, attacker_id, defender_id, monster_name, skill_id, value, is_crit, is_lucky, hp_loss, shield_loss, is_boss)
SELECT id, encounter_id, timestamp_ms, attacker_id, defender_id, monster_name, skill_id, value, is_crit, is_lucky, hp_loss, shield_loss, is_boss FROM damage_events;
DROP TABLE damage_events;
ALTER TABLE damage_events_new RENAME TO damage_events;

-- Recreate encounter_bosses without max_hp
CREATE TABLE encounter_bosses_new (
    encounter_id INTEGER NOT NULL,
    monster_name TEXT NOT NULL,
    hits INTEGER NOT NULL DEFAULT 0,
    total_damage BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (encounter_id, monster_name)
);
INSERT INTO encounter_bosses_new (encounter_id, monster_name, hits, total_damage)
SELECT encounter_id, monster_name, hits, total_damage FROM encounter_bosses;
DROP TABLE encounter_bosses;
ALTER TABLE encounter_bosses_new RENAME TO encounter_bosses;

COMMIT;
PRAGMA foreign_keys=on;
