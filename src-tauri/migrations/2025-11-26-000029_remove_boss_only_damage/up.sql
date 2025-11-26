-- Remove boss-only damage columns from actor_encounter_stats table
ALTER TABLE actor_encounter_stats DROP COLUMN boss_lucky_total_dealt;
ALTER TABLE actor_encounter_stats DROP COLUMN boss_crit_total_dealt;
ALTER TABLE actor_encounter_stats DROP COLUMN boss_lucky_hits_dealt;
ALTER TABLE actor_encounter_stats DROP COLUMN boss_crit_hits_dealt;
ALTER TABLE actor_encounter_stats DROP COLUMN boss_hits_dealt;
ALTER TABLE actor_encounter_stats DROP COLUMN boss_damage_dealt;
