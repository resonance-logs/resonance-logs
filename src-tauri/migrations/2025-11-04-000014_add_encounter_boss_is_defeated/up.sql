-- Add is_defeated column to encounter_bosses
PRAGMA foreign_keys = ON;

ALTER TABLE encounter_bosses ADD COLUMN is_defeated INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_encounter_bosses_is_defeated ON encounter_bosses(is_defeated);
