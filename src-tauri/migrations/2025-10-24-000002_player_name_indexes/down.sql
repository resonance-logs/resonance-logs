-- Remove indexes for player name queries
DROP INDEX IF EXISTS idx_entities_player_last_seen;
DROP INDEX IF EXISTS idx_entities_player_name;