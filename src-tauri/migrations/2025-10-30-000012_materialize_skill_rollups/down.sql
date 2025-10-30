-- Drop the tables created in up.sql
DROP INDEX IF EXISTS idx_encounter_bosses_encounter;
DROP INDEX IF EXISTS idx_heal_skill_stats_healer;
DROP INDEX IF EXISTS idx_heal_skill_stats_encounter;
DROP INDEX IF EXISTS idx_damage_skill_stats_attacker;
DROP INDEX IF EXISTS idx_damage_skill_stats_encounter;

DROP TABLE IF EXISTS encounter_bosses;
DROP TABLE IF EXISTS heal_skill_stats;
DROP TABLE IF EXISTS damage_skill_stats;
