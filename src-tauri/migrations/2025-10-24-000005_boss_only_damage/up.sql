-- Add boss-only damage columns to damage_events table
ALTER TABLE damage_events ADD COLUMN is_boss INTEGER NOT NULL DEFAULT 0;

-- Add boss-only damage columns to actor_encounter_stats table
ALTER TABLE actor_encounter_stats ADD COLUMN boss_damage_dealt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN boss_hits_dealt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN boss_crit_hits_dealt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN boss_lucky_hits_dealt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN boss_crit_total_dealt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN boss_lucky_total_dealt INTEGER NOT NULL DEFAULT 0;

-- Add indexes for boss-only damage queries
CREATE INDEX IF NOT EXISTS idx_damage_events_boss ON damage_events(is_boss);
CREATE INDEX IF NOT EXISTS idx_damage_events_encounter_boss ON damage_events(encounter_id, is_boss);