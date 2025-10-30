-- Add monster_name column to damage_skill_stats
-- This allows tracking which monster was damaged without joining to damage_events
ALTER TABLE damage_skill_stats ADD COLUMN monster_name TEXT;

-- Add monster_name column to heal_skill_stats
-- This allows tracking which monster was healed (rare but possible)
ALTER TABLE heal_skill_stats ADD COLUMN monster_name TEXT;


