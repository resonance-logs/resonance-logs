-- Remove indexes
DROP INDEX IF EXISTS idx_damage_events_boss;
DROP INDEX IF EXISTS idx_damage_events_encounter_boss;

-- Remove boss-only damage columns from actor_encounter_stats table
ALTER TABLE actor_encounter_stats DROP COLUMN boss_lucky_total_dealt;
ALTER TABLE actor_encounter_stats DROP COLUMN boss_crit_total_dealt;
ALTER TABLE actor_encounter_stats DROP COLUMN boss_lucky_hits_dealt;
ALTER TABLE actor_encounter_stats DROP COLUMN boss_crit_hits_dealt;
ALTER TABLE actor_encounter_stats DROP COLUMN boss_hits_dealt;
ALTER TABLE actor_encounter_stats DROP COLUMN boss_damage_dealt;

-- Remove boss-only damage column from damage_events table
ALTER TABLE damage_events DROP COLUMN is_boss;