-- Recreate sessions table and encounters table with session_id foreign key
DROP TABLE IF EXISTS encounters;
DROP TABLE IF EXISTS sessions;

CREATE TABLE sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at_ms INTEGER NOT NULL,
  ended_at_ms INTEGER,
  version TEXT,
  platform TEXT
);

CREATE TABLE encounters (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id INTEGER,
  started_at_ms INTEGER NOT NULL,
  ended_at_ms INTEGER,
  local_player_id INTEGER,
  total_dmg INTEGER DEFAULT 0,
  total_heal INTEGER DEFAULT 0,
  FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE SET NULL
);

-- Recreate the index
CREATE INDEX idx_encounters_started ON encounters(started_at_ms);