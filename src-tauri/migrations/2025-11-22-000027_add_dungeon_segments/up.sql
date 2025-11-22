-- Create dungeon_segments table to store segments for each encounter
CREATE TABLE IF NOT EXISTS dungeon_segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    encounter_id INTEGER NOT NULL,
    segment_type TEXT NOT NULL, -- 'boss' or 'trash'
    boss_entity_id BIGINT,
    boss_monster_type_id BIGINT,
    boss_name TEXT,
    started_at_ms BIGINT NOT NULL,
    ended_at_ms BIGINT,
    total_damage BIGINT NOT NULL DEFAULT 0,
    hit_count BIGINT NOT NULL DEFAULT 0,
    FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

CREATE INDEX idx_dungeon_segments_encounter ON dungeon_segments(encounter_id);
CREATE INDEX idx_dungeon_segments_type ON dungeon_segments(segment_type);
