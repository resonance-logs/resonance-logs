-- Add death_events table for recording player death occurrences
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS death_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL,
  timestamp_ms INTEGER NOT NULL,
  actor_id INTEGER NOT NULL,
  killer_id INTEGER,
  skill_id INTEGER,
  is_local_player INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_death_events_encounter ON death_events(encounter_id);
CREATE INDEX IF NOT EXISTS idx_death_events_actor_time ON death_events(actor_id, timestamp_ms);
