-- Add attempts table for boss splitting feature
-- Each encounter can have multiple attempts (Attempt 1, Attempt 2, etc.)
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS attempts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL,
  attempt_index INTEGER NOT NULL,
  started_at_ms INTEGER NOT NULL,
  ended_at_ms INTEGER,
  reason TEXT NOT NULL, -- 'wipe', 'hp_rollback', 'manual'
  boss_hp_start INTEGER,
  boss_hp_end INTEGER,
  total_deaths INTEGER DEFAULT 0,
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE,
  UNIQUE(encounter_id, attempt_index)
);

CREATE INDEX IF NOT EXISTS idx_attempts_encounter ON attempts(encounter_id);
CREATE INDEX IF NOT EXISTS idx_attempts_encounter_index ON attempts(encounter_id, attempt_index);

-- Add attempt_index to damage_events
ALTER TABLE damage_events ADD COLUMN attempt_index INTEGER DEFAULT 1;
CREATE INDEX IF NOT EXISTS idx_damage_events_attempt ON damage_events(encounter_id, attempt_index);

-- Add attempt_index to heal_events
ALTER TABLE heal_events ADD COLUMN attempt_index INTEGER DEFAULT 1;
CREATE INDEX IF NOT EXISTS idx_heal_events_attempt ON heal_events(encounter_id, attempt_index);

-- Add attempt_index to death_events
ALTER TABLE death_events ADD COLUMN attempt_index INTEGER DEFAULT 1;
CREATE INDEX IF NOT EXISTS idx_death_events_attempt ON death_events(encounter_id, attempt_index);

-- Create initial attempt for all existing encounters
INSERT INTO attempts (encounter_id, attempt_index, started_at_ms, ended_at_ms, reason, total_deaths)
SELECT
  id as encounter_id,
  1 as attempt_index,
  started_at_ms,
  ended_at_ms,
  'manual' as reason,
  (SELECT COUNT(*) FROM death_events WHERE death_events.encounter_id = encounters.id) as total_deaths
FROM encounters;
