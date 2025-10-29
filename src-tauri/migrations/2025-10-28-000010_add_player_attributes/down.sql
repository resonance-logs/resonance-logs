-- Rollback migration: Remove player attributes columns
-- WARNING: This will permanently delete all stored player attribute data

-- Drop index
DROP INDEX IF EXISTS idx_entities_attributes;

-- Remove attributes column from actor_encounter_stats
-- SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
CREATE TABLE actor_encounter_stats_backup (
    encounter_id INTEGER NOT NULL,
    actor_id INTEGER NOT NULL,
    name TEXT,
    class_id INTEGER,
    ability_score INTEGER,
    level INTEGER,
    damage_dealt INTEGER NOT NULL DEFAULT 0,
    heal_dealt INTEGER NOT NULL DEFAULT 0,
    damage_taken INTEGER NOT NULL DEFAULT 0,
    hits_dealt INTEGER NOT NULL DEFAULT 0,
    hits_heal INTEGER NOT NULL DEFAULT 0,
    hits_taken INTEGER NOT NULL DEFAULT 0,
    crit_hits_dealt INTEGER NOT NULL DEFAULT 0,
    crit_hits_heal INTEGER NOT NULL DEFAULT 0,
    crit_hits_taken INTEGER NOT NULL DEFAULT 0,
    lucky_hits_dealt INTEGER NOT NULL DEFAULT 0,
    lucky_hits_heal INTEGER NOT NULL DEFAULT 0,
    lucky_hits_taken INTEGER NOT NULL DEFAULT 0,
    crit_total_dealt INTEGER NOT NULL DEFAULT 0,
    crit_total_heal INTEGER NOT NULL DEFAULT 0,
    crit_total_taken INTEGER NOT NULL DEFAULT 0,
    lucky_total_dealt INTEGER NOT NULL DEFAULT 0,
    lucky_total_heal INTEGER NOT NULL DEFAULT 0,
    lucky_total_taken INTEGER NOT NULL DEFAULT 0,
    boss_damage_dealt INTEGER NOT NULL DEFAULT 0,
    boss_hits_dealt INTEGER NOT NULL DEFAULT 0,
    boss_crit_hits_dealt INTEGER NOT NULL DEFAULT 0,
    boss_lucky_hits_dealt INTEGER NOT NULL DEFAULT 0,
    boss_crit_total_dealt INTEGER NOT NULL DEFAULT 0,
    boss_lucky_total_dealt INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (encounter_id, actor_id)
);

INSERT INTO actor_encounter_stats_backup SELECT
    encounter_id, actor_id, name, class_id, ability_score, level,
    damage_dealt, heal_dealt, damage_taken,
    hits_dealt, hits_heal, hits_taken,
    crit_hits_dealt, crit_hits_heal, crit_hits_taken,
    lucky_hits_dealt, lucky_hits_heal, lucky_hits_taken,
    crit_total_dealt, crit_total_heal, crit_total_taken,
    lucky_total_dealt, lucky_total_heal, lucky_total_taken,
    boss_damage_dealt, boss_hits_dealt, boss_crit_hits_dealt,
    boss_lucky_hits_dealt, boss_crit_total_dealt, boss_lucky_total_dealt
FROM actor_encounter_stats;

DROP TABLE actor_encounter_stats;
ALTER TABLE actor_encounter_stats_backup RENAME TO actor_encounter_stats;

-- Remove attributes column from entities
CREATE TABLE entities_backup (
    entity_id INTEGER PRIMARY KEY,
    name TEXT,
    class_id INTEGER,
    class_spec INTEGER,
    ability_score INTEGER,
    level INTEGER,
    first_seen_ms INTEGER,
    last_seen_ms INTEGER
);

INSERT INTO entities_backup SELECT
    entity_id, name, class_id, class_spec, ability_score, level, first_seen_ms, last_seen_ms
FROM entities;

DROP TABLE entities;
ALTER TABLE entities_backup RENAME TO entities;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_entities_last_seen ON entities(last_seen_ms);
