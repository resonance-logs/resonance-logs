-- Create the buffs table for storing buff event history
CREATE TABLE buffs (
    encounter_id INTEGER NOT NULL,
    entity_id BIGINT NOT NULL,
    buff_id INTEGER NOT NULL,
    events TEXT NOT NULL, -- JSON array of buff events
    PRIMARY KEY (encounter_id, entity_id, buff_id),
    FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

-- Index for faster lookup by encounter
CREATE INDEX idx_buffs_encounter_id ON buffs(encounter_id);
