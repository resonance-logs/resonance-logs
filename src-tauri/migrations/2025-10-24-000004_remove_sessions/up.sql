-- Remove sessions table and session_id from encounters table
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS encounters;

CREATE TABLE encounters (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at_ms INTEGER NOT NULL,
  ended_at_ms INTEGER,
  local_player_id INTEGER,
  total_dmg INTEGER DEFAULT 0,
  total_heal INTEGER DEFAULT 0
);

-- Recreate the index
CREATE INDEX idx_encounters_started ON encounters(started_at_ms);