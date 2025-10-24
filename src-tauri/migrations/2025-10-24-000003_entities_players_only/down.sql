-- Recreate original entities table with entity_type and is_player columns
DROP TABLE IF EXISTS entities;

CREATE TABLE entities (
  entity_id INTEGER PRIMARY KEY,
  entity_type INTEGER NOT NULL,
  is_player INTEGER NOT NULL DEFAULT 0,
  name TEXT,
  class_id INTEGER,
  class_spec INTEGER,
  ability_score INTEGER,
  level INTEGER,
  first_seen_ms INTEGER,
  last_seen_ms INTEGER
);

-- Recreate original indexes
CREATE INDEX idx_entities_last_seen ON entities(last_seen_ms);
CREATE INDEX idx_entities_player_last_seen ON entities(is_player, last_seen_ms DESC) WHERE is_player = 1;
CREATE INDEX idx_entities_player_name ON entities(entity_id) WHERE is_player = 1 AND name IS NOT NULL;