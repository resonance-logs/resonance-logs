-- Add defender_name column to damage_events for recording monster names captured from packets
ALTER TABLE damage_events ADD COLUMN defender_name TEXT;

-- Backfill existing rows best-effort: mark non-player defenders (not present in entities table) as Unknown Monster
-- We cannot recover historical monster names from packets for past events, so set a generic placeholder
UPDATE damage_events
SET defender_name = 'Unknown Monster'
WHERE defender_id IS NOT NULL
  AND defender_name IS NULL
  AND defender_id NOT IN (SELECT entity_id FROM entities);

-- Optional index to support filtering by defender_name (if needed by UI)
CREATE INDEX IF NOT EXISTS idx_damage_events_defender_name ON damage_events(defender_name);
