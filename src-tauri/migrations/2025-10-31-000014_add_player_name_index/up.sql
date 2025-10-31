-- Add index on actor_encounter_stats(name) for faster player name autocomplete queries
-- This index improves LIKE pattern matching performance for player name filtering
CREATE INDEX idx_actor_stats_name ON actor_encounter_stats(name) WHERE is_player = 1 AND name IS NOT NULL;
