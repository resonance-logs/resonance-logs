-- Add defender_max_hp to damage_events and max_hp to encounter_bosses
ALTER TABLE damage_events ADD COLUMN defender_max_hp BIGINT NULL;

ALTER TABLE encounter_bosses ADD COLUMN max_hp BIGINT NULL;

-- Optional: create an index for quicker queries
CREATE INDEX IF NOT EXISTS idx_encounter_bosses_max_hp ON encounter_bosses(max_hp);
