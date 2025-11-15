-- Add actor_phase_stats table for per-phase actor statistics
-- Similar to actor_encounter_stats but scoped to individual phases
PRAGMA foreign_keys = ON;

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
