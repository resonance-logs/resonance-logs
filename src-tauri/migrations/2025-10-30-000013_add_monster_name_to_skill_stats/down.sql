-- Remove monster_name column from heal_skill_stats
CREATE TABLE heal_skill_stats_new (
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
    heal_details TEXT NOT NULL,
    PRIMARY KEY (encounter_id, healer_id, target_id, skill_id),
    FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

INSERT INTO heal_skill_stats_new SELECT encounter_id, healer_id, target_id, skill_id, hits, total_value, crit_hits, lucky_hits, crit_total, lucky_total, heal_details FROM heal_skill_stats;
DROP TABLE heal_skill_stats;
ALTER TABLE heal_skill_stats_new RENAME TO heal_skill_stats;

CREATE INDEX idx_heal_skill_stats_encounter ON heal_skill_stats(encounter_id);
CREATE INDEX idx_heal_skill_stats_healer ON heal_skill_stats(healer_id);

-- Recreate damage_skill_stats without monster_name
CREATE TABLE damage_skill_stats_new (
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
    hit_details TEXT NOT NULL,
    PRIMARY KEY (encounter_id, attacker_id, defender_id, skill_id),
    FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
);

INSERT INTO damage_skill_stats_new SELECT encounter_id, attacker_id, defender_id, skill_id, hits, total_value, crit_hits, lucky_hits, crit_total, lucky_total, hp_loss_total, shield_loss_total, hit_details FROM damage_skill_stats;
DROP TABLE damage_skill_stats;
ALTER TABLE damage_skill_stats_new RENAME TO damage_skill_stats;

CREATE INDEX idx_damage_skill_stats_encounter ON damage_skill_stats(encounter_id);
CREATE INDEX idx_damage_skill_stats_attacker ON damage_skill_stats(attacker_id);
