-- Add boss-only damage columns back to actor_encounter_stats table
ALTER TABLE actor_encounter_stats ADD COLUMN boss_damage_dealt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN boss_hits_dealt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN boss_crit_hits_dealt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN boss_lucky_hits_dealt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN boss_crit_total_dealt INTEGER NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN boss_lucky_total_dealt INTEGER NOT NULL DEFAULT 0;
