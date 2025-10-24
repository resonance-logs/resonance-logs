-- Recreate entities table to only contain players (no entity_type or is_player columns)
DROP TABLE IF EXISTS entities;

CREATE TABLE entities (
  entity_id INTEGER PRIMARY KEY,
  name TEXT,
  class_id INTEGER,
  class_spec INTEGER,
  ability_score INTEGER,
  level INTEGER,
  first_seen_ms INTEGER,
  last_seen_ms INTEGER
);

-- Create indexes
CREATE INDEX idx_entities_last_seen ON entities(last_seen_ms);
CREATE INDEX idx_entities_player_last_seen ON entities(last_seen_ms DESC);
CREATE INDEX idx_entities_player_name ON entities(entity_id) WHERE name IS NOT NULL;