-- Revert adding is_player column
ALTER TABLE actor_encounter_stats DROP COLUMN is_player;
