-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Sessions can be used later to group encounters across app runs
CREATE TABLE IF NOT EXISTS sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at_ms INTEGER NOT NULL,
  ended_at_ms INTEGER,
  version TEXT,
  platform TEXT
);

-- All known entities (players, monsters, etc.) keyed by stable character/entity id
CREATE TABLE IF NOT EXISTS entities (
  entity_id INTEGER PRIMARY KEY,
  entity_type INTEGER NOT NULL,
  is_player INTEGER NOT NULL DEFAULT 0,
  name TEXT,
  class_id INTEGER,
  class_spec INTEGER,
  ability_score INTEGER,
  level INTEGER,
  first_seen_ms INTEGER,
  last_seen_ms INTEGER
);
CREATE INDEX IF NOT EXISTS idx_entities_last_seen ON entities(last_seen_ms);

-- Encounter per fight
CREATE TABLE IF NOT EXISTS encounters (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id INTEGER,
  started_at_ms INTEGER NOT NULL,
  ended_at_ms INTEGER,
  local_player_id INTEGER,
  total_dmg INTEGER DEFAULT 0,
  total_heal INTEGER DEFAULT 0,
  FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_encounters_started ON encounters(started_at_ms);

-- Damage events (raw)
CREATE TABLE IF NOT EXISTS damage_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL,
  timestamp_ms INTEGER NOT NULL,
  attacker_id INTEGER NOT NULL,
  defender_id INTEGER,
  skill_id INTEGER,
  value INTEGER NOT NULL,
  is_crit INTEGER NOT NULL DEFAULT 0,
  is_lucky INTEGER NOT NULL DEFAULT 0,
  hp_loss INTEGER NOT NULL DEFAULT 0,
  shield_loss INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_damage_events_encounter ON damage_events(encounter_id);
CREATE INDEX IF NOT EXISTS idx_damage_events_attacker_time ON damage_events(attacker_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_damage_events_defender_time ON damage_events(defender_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_damage_events_skill ON damage_events(skill_id);

-- Heal events (raw)
CREATE TABLE IF NOT EXISTS heal_events (
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
CREATE INDEX IF NOT EXISTS idx_heal_events_encounter ON heal_events(encounter_id);
CREATE INDEX IF NOT EXISTS idx_heal_events_healer_time ON heal_events(healer_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_heal_events_target_time ON heal_events(target_id, timestamp_ms);
CREATE INDEX IF NOT EXISTS idx_heal_events_skill ON heal_events(skill_id);
