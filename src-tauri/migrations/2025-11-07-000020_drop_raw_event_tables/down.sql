-- Recreate minimal schema for damage_events and heal_events (does NOT restore data)
-- This is best-effort and intended only to allow migration rollbacks to re-create the tables' structure.

CREATE TABLE IF NOT EXISTS damage_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL,
  timestamp_ms BIGINT NOT NULL,
  attacker_id BIGINT NOT NULL,
  defender_id BIGINT,
  monster_name TEXT,
  skill_id INTEGER,
  value BIGINT NOT NULL,
  is_crit INTEGER NOT NULL DEFAULT 0,
  is_lucky INTEGER NOT NULL DEFAULT 0,
  hp_loss BIGINT NOT NULL DEFAULT 0,
  shield_loss BIGINT NOT NULL DEFAULT 0,
  defender_max_hp BIGINT,
  is_boss INTEGER NOT NULL DEFAULT 0,
  attempt_index INTEGER DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_damage_events_encounter ON damage_events(encounter_id);
CREATE INDEX IF NOT EXISTS idx_damage_events_attacker_time ON damage_events(attacker_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_damage_events_defender_time ON damage_events(defender_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_damage_events_skill ON damage_events(skill_id);
CREATE INDEX IF NOT EXISTS idx_damage_events_boss ON damage_events(is_boss);
CREATE INDEX IF NOT EXISTS idx_damage_events_encounter_boss ON damage_events(encounter_id, is_boss);
CREATE INDEX IF NOT EXISTS idx_damage_events_monster_name ON damage_events(monster_name);
CREATE INDEX IF NOT EXISTS idx_damage_events_attempt ON damage_events(encounter_id, attempt_index);

CREATE TABLE IF NOT EXISTS heal_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL,
  timestamp_ms BIGINT NOT NULL,
  healer_id BIGINT NOT NULL,
  target_id BIGINT,
  skill_id INTEGER,
  value BIGINT NOT NULL,
  is_crit INTEGER NOT NULL DEFAULT 0,
  is_lucky INTEGER NOT NULL DEFAULT 0,
  attempt_index INTEGER DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_heal_events_encounter ON heal_events(encounter_id);
CREATE INDEX IF NOT EXISTS idx_heal_events_healer_time ON heal_events(healer_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_heal_events_target_time ON heal_events(target_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_heal_events_skill ON heal_events(skill_id);
CREATE INDEX IF NOT EXISTS idx_heal_events_attempt ON heal_events(encounter_id, attempt_index);
