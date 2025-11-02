-- Add is_local_player column to actor_encounter_stats to track which actor is the local player
-- This allows the frontend to display "You" indicator in the history page
ALTER TABLE actor_encounter_stats ADD COLUMN is_local_player INTEGER NOT NULL DEFAULT 0;
