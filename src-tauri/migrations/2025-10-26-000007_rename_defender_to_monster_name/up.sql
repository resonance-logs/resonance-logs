-- Rename defender_name to monster_name and adjust index
-- SQLite 3.25+ supports RENAME COLUMN
-- Drop old index if it exists
DROP INDEX IF EXISTS idx_damage_events_defender_name;

-- Rename column
ALTER TABLE damage_events RENAME COLUMN defender_name TO monster_name;

-- Create new index on monster_name
CREATE INDEX IF NOT EXISTS idx_damage_events_monster_name ON damage_events(monster_name);
