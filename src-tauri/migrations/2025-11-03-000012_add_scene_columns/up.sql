-- Add scene fields to encounters table
ALTER TABLE encounters ADD COLUMN scene_id INTEGER;
ALTER TABLE encounters ADD COLUMN scene_name TEXT;

-- Optional indexes to speed queries filtering by scene
CREATE INDEX IF NOT EXISTS idx_encounters_scene_id ON encounters(scene_id);
CREATE INDEX IF NOT EXISTS idx_encounters_scene_name ON encounters(scene_name);
