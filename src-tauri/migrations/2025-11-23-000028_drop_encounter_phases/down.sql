-- Recreate phase tables (rollback migration)
-- This migration cannot be fully reversed as data is already lost

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

CREATE TABLE IF NOT EXISTS actor_phase_stats (
  phase_id INTEGER NOT NULL,
  actor_id INTEGER NOT NULL,
  name TEXT,
  class_id INTEGER,
  class_spec INTEGER,
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
  revives INTEGER NOT NULL DEFAULT 0,
  is_player INTEGER NOT NULL DEFAULT 0,
  is_local_player INTEGER NOT NULL DEFAULT 0,
  attributes TEXT,
  PRIMARY KEY (phase_id, actor_id),
  FOREIGN KEY(phase_id) REFERENCES encounter_phases(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_actor_phase_stats_phase ON actor_phase_stats(phase_id);
CREATE INDEX IF NOT EXISTS idx_actor_phase_stats_actor ON actor_phase_stats(actor_id);
CREATE INDEX IF NOT EXISTS idx_actor_phase_stats_player ON actor_phase_stats(is_player);
