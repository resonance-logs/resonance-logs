ALTER TABLE actor_encounter_stats ADD COLUMN active_dmg_time_ms BIGINT NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN tdps DOUBLE NOT NULL DEFAULT 0;
