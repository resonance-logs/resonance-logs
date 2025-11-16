DROP INDEX IF EXISTS idx_encounters_uploaded_at;

CREATE TABLE encounters_backup AS
SELECT
    id,
    started_at_ms,
    ended_at_ms,
    local_player_id,
    total_dmg,
    total_heal,
    scene_id,
    scene_name,
    duration
FROM encounters;

DROP TABLE encounters;

CREATE TABLE encounters (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    started_at_ms BIGINT NOT NULL,
    ended_at_ms BIGINT,
    local_player_id BIGINT,
    total_dmg BIGINT,
    total_heal BIGINT,
    scene_id INTEGER,
    scene_name TEXT,
    duration DOUBLE NOT NULL DEFAULT 0
);

INSERT INTO encounters
SELECT
    id,
    started_at_ms,
    ended_at_ms,
    local_player_id,
    total_dmg,
    total_heal,
    scene_id,
    scene_name,
    duration
FROM encounters_backup;

DROP TABLE encounters_backup;
