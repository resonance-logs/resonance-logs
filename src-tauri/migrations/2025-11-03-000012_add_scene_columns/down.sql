-- Remove scene fields by recreating encounters table without them.
-- WARNING: This will drop the scene_id and scene_name columns and must be used with care.
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TABLE encounters_tmp (
  id INTEGER PRIMARY KEY,
  started_at_ms BIGINT NOT NULL,
  ended_at_ms BIGINT NULL,
  local_player_id BIGINT NULL,
  total_dmg BIGINT NULL,
  total_heal BIGINT NULL
  -- add other existing columns here exactly as they are in your current schema
);

INSERT INTO encounters_tmp (id, started_at_ms, ended_at_ms, local_player_id, total_dmg, total_heal)
SELECT id, started_at_ms, ended_at_ms, local_player_id, total_dmg, total_heal FROM encounters;

DROP TABLE encounters;
ALTER TABLE encounters_tmp RENAME TO encounters;
COMMIT;
PRAGMA foreign_keys = ON;
