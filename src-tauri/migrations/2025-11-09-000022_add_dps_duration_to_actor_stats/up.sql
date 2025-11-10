-- Add DPS and encounter duration snapshots to actor encounter stats
ALTER TABLE actor_encounter_stats ADD COLUMN dps REAL NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN duration REAL NOT NULL DEFAULT 0;

-- Backfill historical records using encounter start/end timestamps (players only)
-- Use a robust expression that never yields NULL: if encounter timing is missing, default to 1.0s
UPDATE actor_encounter_stats
SET
    duration = COALESCE((
        SELECT CASE
            WHEN e.ended_at_ms IS NOT NULL AND e.ended_at_ms > e.started_at_ms THEN ((e.ended_at_ms - e.started_at_ms) / 1000.0)
            ELSE 1.0
        END
        FROM encounters e
        WHERE e.id = actor_encounter_stats.encounter_id
    ), 1.0),
    dps = CASE
        WHEN COALESCE((
            SELECT CASE
                WHEN e.ended_at_ms IS NOT NULL AND e.ended_at_ms > e.started_at_ms THEN ((e.ended_at_ms - e.started_at_ms) / 1000.0)
                ELSE 1.0
            END
            FROM encounters e
            WHERE e.id = actor_encounter_stats.encounter_id
        ), 1.0) > 0
        THEN damage_dealt * 1.0 /
            COALESCE((
                SELECT CASE
                    WHEN e.ended_at_ms IS NOT NULL AND e.ended_at_ms > e.started_at_ms THEN ((e.ended_at_ms - e.started_at_ms) / 1000.0)
                    ELSE 1.0
                END
                FROM encounters e
                WHERE e.id = actor_encounter_stats.encounter_id
            ), 1.0)
        ELSE 0
    END
WHERE is_player = 1;

-- Add encounter duration snapshot at the encounter level
ALTER TABLE encounters ADD COLUMN duration REAL NOT NULL DEFAULT 0;

-- Backfill encounter duration for completed encounters
UPDATE encounters
SET duration = CASE
    WHEN ended_at_ms IS NOT NULL AND ended_at_ms > started_at_ms THEN (ended_at_ms - started_at_ms) / 1000.0
    ELSE duration
END;
