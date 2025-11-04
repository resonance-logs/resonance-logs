-- Recreate encounter_bosses without is_defeated
PRAGMA foreign_keys = ON;

CREATE TABLE encounter_bosses_new (
  encounter_id INTEGER NOT NULL,
  monster_name TEXT NOT NULL,
  hits INTEGER NOT NULL DEFAULT 0,
  total_damage INTEGER NOT NULL DEFAULT 0,
  max_hp BIGINT NULL,
  PRIMARY KEY (encounter_id, monster_name),
  FOREIGN KEY(encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

INSERT INTO encounter_bosses_new (encounter_id, monster_name, hits, total_damage, max_hp)
SELECT encounter_id, monster_name, hits, total_damage, max_hp FROM encounter_bosses;

DROP TABLE encounter_bosses;
ALTER TABLE encounter_bosses_new RENAME TO encounter_bosses;
