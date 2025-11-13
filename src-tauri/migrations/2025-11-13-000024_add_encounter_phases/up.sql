-- Add encounter_phases table for boss encounter splitting
-- Each encounter can have multiple phases (mob phase, boss phase)
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS encounter_phases (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  encounter_id INTEGER NOT NULL,
  phase_type TEXT NOT NULL CHECK(phase_type IN ('mob', 'boss')),
  start_time_ms INTEGER NOT NULL,
  end_time_ms INTEGER,
  outcome TEXT NOT NULL DEFAULT 'unknown' CHECK(outcome IN ('success', 'wipe', 'unknown')),
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_encounter_phases_encounter ON encounter_phases(encounter_id);
CREATE INDEX IF NOT EXISTS idx_encounter_phases_type ON encounter_phases(phase_type);
CREATE INDEX IF NOT EXISTS idx_encounter_phases_outcome ON encounter_phases(outcome);
