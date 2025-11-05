-- Remove attempts table and related columns
PRAGMA foreign_keys = OFF;

-- Drop indexes
DROP INDEX IF EXISTS idx_attempts_encounter;
DROP INDEX IF EXISTS idx_attempts_encounter_index;
DROP INDEX IF EXISTS idx_damage_events_attempt;
DROP INDEX IF EXISTS idx_heal_events_attempt;
DROP INDEX IF EXISTS idx_death_events_attempt;

-- Recreate damage_events without attempt_index
CREATE TABLE damage_events_tmp (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL,
  timestamp_ms INTEGER NOT NULL,
  attacker_id INTEGER NOT NULL,
  defender_id INTEGER,
  monster_name TEXT,
  skill_id INTEGER,
  value INTEGER NOT NULL,
  is_crit INTEGER NOT NULL DEFAULT 0,
  is_lucky INTEGER NOT NULL DEFAULT 0,
  hp_loss INTEGER NOT NULL DEFAULT 0,
  shield_loss INTEGER NOT NULL DEFAULT 0,
  defender_max_hp INTEGER,
  is_boss INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

INSERT INTO damage_events_tmp
SELECT id, encounter_id, timestamp_ms, attacker_id, defender_id, monster_name,
       skill_id, value, is_crit, is_lucky, hp_loss, shield_loss, defender_max_hp, is_boss
FROM damage_events;

DROP TABLE damage_events;
ALTER TABLE damage_events_tmp RENAME TO damage_events;

-- Recreate heal_events without attempt_index
CREATE TABLE heal_events_tmp (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL,
  timestamp_ms INTEGER NOT NULL,
  healer_id INTEGER NOT NULL,
  target_id INTEGER,
  skill_id INTEGER,
  value INTEGER NOT NULL,
  is_crit INTEGER NOT NULL DEFAULT 0,
  is_lucky INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

INSERT INTO heal_events_tmp
SELECT id, encounter_id, timestamp_ms, healer_id, target_id, skill_id, value, is_crit, is_lucky
FROM heal_events;

DROP TABLE heal_events;
ALTER TABLE heal_events_tmp RENAME TO heal_events;

-- Recreate death_events without attempt_index
CREATE TABLE death_events_tmp (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL,
  timestamp_ms INTEGER NOT NULL,
  actor_id INTEGER NOT NULL,
  killer_id INTEGER,
  skill_id INTEGER,
  is_local_player INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

INSERT INTO death_events_tmp
SELECT id, encounter_id, timestamp_ms, actor_id, killer_id, skill_id, is_local_player
FROM death_events;

DROP TABLE death_events;
ALTER TABLE death_events_tmp RENAME TO death_events;

-- Drop attempts table
DROP TABLE IF EXISTS attempts;

-- Recreate indexes for event tables
CREATE INDEX IF NOT EXISTS idx_damage_events_encounter ON damage_events(encounter_id);
CREATE INDEX IF NOT EXISTS idx_damage_events_attacker_time ON damage_events(attacker_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_damage_events_defender_time ON damage_events(defender_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_damage_events_skill ON damage_events(skill_id);

CREATE INDEX IF NOT EXISTS idx_heal_events_encounter ON heal_events(encounter_id);
CREATE INDEX IF NOT EXISTS idx_heal_events_healer_time ON heal_events(healer_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_heal_events_target_time ON heal_events(target_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_heal_events_skill ON heal_events(skill_id);

CREATE INDEX IF NOT EXISTS idx_death_events_encounter ON death_events(encounter_id);
CREATE INDEX IF NOT EXISTS idx_death_events_actor_time ON death_events(actor_id, timestamp_ms);

PRAGMA foreign_keys = ON;
