-- Add indexes for optimal player name query performance
CREATE INDEX idx_entities_player_last_seen ON entities(is_player, last_seen_ms DESC) WHERE is_player = 1;
CREATE INDEX idx_entities_player_name ON entities(entity_id) WHERE is_player = 1 AND name IS NOT NULL;