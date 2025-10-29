-- Migration: Add player attributes JSON column to entities and actor_encounter_stats
-- BREAKING: This migration adds new columns for extended player attribute storage.
-- It is backwards-incompatible and requires a database reset for older versions.

-- Add attributes JSON column to entities table
ALTER TABLE entities ADD COLUMN attributes TEXT;

-- Add attributes JSON column to actor_encounter_stats table
ALTER TABLE actor_encounter_stats ADD COLUMN attributes TEXT;

-- Create index on entities.attributes for potential JSON queries (optional but recommended)
-- Note: SQLite JSON support is limited, but this prepares for future optimizations
CREATE INDEX IF NOT EXISTS idx_entities_attributes ON entities(attributes);
