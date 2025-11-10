CREATE TABLE detailed_playerdata (
    player_id      BIGINT PRIMARY KEY,
    last_seen_ms   BIGINT NOT NULL,
    char_serialize_json   TEXT NOT NULL,
    profession_list_json  TEXT,
    talent_node_ids_json  TEXT
);

CREATE INDEX idx_detailed_playerdata_last_seen
    ON detailed_playerdata(last_seen_ms);
