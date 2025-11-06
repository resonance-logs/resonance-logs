-- Add revive count column to actor_encounter_stats

ALTER TABLE actor_encounter_stats ADD COLUMN revives INTEGER NOT NULL DEFAULT 0;
