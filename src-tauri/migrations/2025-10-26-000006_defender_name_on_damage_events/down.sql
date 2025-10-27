-- Drop index if exists
DROP INDEX IF EXISTS idx_damage_events_defender_name;

-- Remove defender_name column from damage_events
ALTER TABLE damage_events DROP COLUMN defender_name;