-- Remove snapshot columns from actor_encounter_stats
ALTER TABLE actor_encounter_stats DROP COLUMN name;
ALTER TABLE actor_encounter_stats DROP COLUMN class_id;
ALTER TABLE actor_encounter_stats DROP COLUMN ability_score;
ALTER TABLE actor_encounter_stats DROP COLUMN level;
