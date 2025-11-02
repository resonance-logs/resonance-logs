-- Remove is_local_player column from actor_encounter_stats
ALTER TABLE actor_encounter_stats DROP COLUMN is_local_player;
