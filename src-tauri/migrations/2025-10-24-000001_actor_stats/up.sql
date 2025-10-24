-- Per-encounter, per-actor aggregated stats (materialized for fast queries)
CREATE TABLE IF NOT EXISTS actor_encounter_stats (
  encounter_id INTEGER NOT NULL,
  actor_id INTEGER NOT NULL,
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
  PRIMARY KEY (encounter_id, actor_id),
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_actor_stats_encounter ON actor_encounter_stats(encounter_id);
