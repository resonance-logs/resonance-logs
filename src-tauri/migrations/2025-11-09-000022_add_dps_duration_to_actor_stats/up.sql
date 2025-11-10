-- Add DPS and encounter duration snapshots to actor encounter stats
ALTER TABLE actor_encounter_stats ADD COLUMN dps REAL NOT NULL DEFAULT 0;
ALTER TABLE actor_encounter_stats ADD COLUMN duration REAL NOT NULL DEFAULT 0;

-- Backfill historical records using encounter start/end timestamps (players only)
UPDATE actor_encounter_stats
SET
    duration = (
        SELECT MAX(((e.ended_at_ms - e.started_at_ms) / 1000.0), 1.0)
        FROM encounters e
        WHERE e.id = actor_encounter_stats.encounter_id
    ),
    dps = CASE
        WHEN (
            SELECT MAX(((e.ended_at_ms - e.started_at_ms) / 1000.0), 1.0)
            FROM encounters e
            WHERE e.id = actor_encounter_stats.encounter_id
        ) > 0
        THEN damage_dealt * 1.0 /
            (
                SELECT MAX(((e.ended_at_ms - e.started_at_ms) / 1000.0), 1.0)
                FROM encounters e
                WHERE e.id = actor_encounter_stats.encounter_id
            )
        ELSE 0
    }
WHERE is_player = 1;
