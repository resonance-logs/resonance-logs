-- Add remote_encounter_id column to encounters table
-- This stores the encounter ID from the public website after successful upload
ALTER TABLE encounters ADD COLUMN remote_encounter_id BIGINT;

-- Add index for faster lookups
CREATE INDEX IF NOT EXISTS idx_encounters_remote_id ON encounters(remote_encounter_id);
