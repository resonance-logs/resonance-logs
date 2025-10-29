-- Add is_player column to actor_encounter_stats to distinguish players from monsters
-- Default to 1 (true) for existing rows since historically only players were tracked
ALTER TABLE actor_encounter_stats ADD COLUMN is_player INTEGER NOT NULL DEFAULT 1;
