-- Create damage_skill_stats table to store aggregated damage per skill/actor/defender
CREATE TABLE damage_skill_stats (
    encounter_id INTEGER NOT NULL,
    attacker_id BIGINT NOT NULL,
    defender_id BIGINT,
    skill_id INTEGER NOT NULL,
    hits INTEGER NOT NULL DEFAULT 0,
    total_value BIGINT NOT NULL DEFAULT 0,
    crit_hits INTEGER NOT NULL DEFAULT 0,
    lucky_hits INTEGER NOT NULL DEFAULT 0,
    crit_total BIGINT NOT NULL DEFAULT 0,
    lucky_total BIGINT NOT NULL DEFAULT 0,
    hp_loss_total BIGINT NOT NULL DEFAULT 0,
    shield_loss_total BIGINT NOT NULL DEFAULT 0,
    -- JSON array storing individual hit details: [{t: timestamp_ms, v: value, c: is_crit, l: is_lucky, h: hp_loss, s: shield_loss}]
    hit_details TEXT NOT NULL,
    PRIMARY KEY (encounter_id, attacker_id, defender_id, skill_id),
    FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

-- Create heal_skill_stats table to store aggregated heals per skill/healer/target
CREATE TABLE heal_skill_stats (
    encounter_id INTEGER NOT NULL,
    healer_id BIGINT NOT NULL,
    target_id BIGINT,
    skill_id INTEGER NOT NULL,
    hits INTEGER NOT NULL DEFAULT 0,
    total_value BIGINT NOT NULL DEFAULT 0,
    crit_hits INTEGER NOT NULL DEFAULT 0,
    lucky_hits INTEGER NOT NULL DEFAULT 0,
    crit_total BIGINT NOT NULL DEFAULT 0,
    lucky_total BIGINT NOT NULL DEFAULT 0,
    -- JSON array storing individual heal details: [{t: timestamp_ms, v: value, c: is_crit, l: is_lucky}]
    heal_details TEXT NOT NULL,
    PRIMARY KEY (encounter_id, healer_id, target_id, skill_id),
    FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

-- Create encounter_bosses table to store boss information per encounter
CREATE TABLE encounter_bosses (
    encounter_id INTEGER NOT NULL,
    monster_name TEXT NOT NULL,
    hits INTEGER NOT NULL DEFAULT 0,
    total_damage BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (encounter_id, monster_name),
    FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

-- Create indexes for efficient querying
CREATE INDEX idx_damage_skill_stats_encounter ON damage_skill_stats(encounter_id);
CREATE INDEX idx_damage_skill_stats_attacker ON damage_skill_stats(attacker_id);
CREATE INDEX idx_heal_skill_stats_encounter ON heal_skill_stats(encounter_id);
CREATE INDEX idx_heal_skill_stats_healer ON heal_skill_stats(healer_id);
CREATE INDEX idx_encounter_bosses_encounter ON encounter_bosses(encounter_id);
