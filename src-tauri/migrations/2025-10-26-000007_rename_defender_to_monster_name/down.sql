-- Revert monster_name back to defender_name and adjust index
DROP INDEX IF EXISTS idx_damage_events_monster_name;

ALTER TABLE damage_events RENAME COLUMN monster_name TO defender_name;

CREATE INDEX IF NOT EXISTS idx_damage_events_defender_name ON damage_events(defender_name);
