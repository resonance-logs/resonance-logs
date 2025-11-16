-- Remove encounter_phases table
PRAGMA foreign_keys = OFF;

DROP INDEX IF EXISTS idx_encounter_phases_encounter;
DROP INDEX IF EXISTS idx_encounter_phases_type;
DROP INDEX IF EXISTS idx_encounter_phases_outcome;

DROP TABLE IF EXISTS encounter_phases;

PRAGMA foreign_keys = ON;
