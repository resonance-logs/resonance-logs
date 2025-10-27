-- Add snapshot columns to actor_encounter_stats for capturing actor state at encounter time
ALTER TABLE actor_encounter_stats ADD COLUMN name TEXT NULL;
ALTER TABLE actor_encounter_stats ADD COLUMN class_id INTEGER NULL;
ALTER TABLE actor_encounter_stats ADD COLUMN ability_score INTEGER NULL;
ALTER TABLE actor_encounter_stats ADD COLUMN level INTEGER NULL;
