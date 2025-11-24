-- Drop encounter_phases and actor_phase_stats tables
-- Segments replace phases for encounter tracking

PRAGMA foreign_keys = OFF;

DROP TABLE IF EXISTS actor_phase_stats;
DROP TABLE IF EXISTS encounter_phases;

PRAGMA foreign_keys = ON;
