// @generated automatically by Diesel CLI, but maintained manually here.
// Keep in sync with migrations.



diesel::table! {
    entities (entity_id) {
        entity_id -> BigInt,
        name -> Nullable<Text>,
        class_id -> Nullable<Integer>,
        class_spec -> Nullable<Integer>,
        ability_score -> Nullable<Integer>,
        level -> Nullable<Integer>,
        first_seen_ms -> Nullable<BigInt>,
        last_seen_ms -> Nullable<BigInt>,
    }
}

diesel::table! {
    skills (skill_id) {
        skill_id -> Integer,
        name -> Nullable<Text>,
    }
}

diesel::table! {
    encounters (id) {
        id -> Integer,
        started_at_ms -> BigInt,
        ended_at_ms -> Nullable<BigInt>,
        local_player_id -> Nullable<BigInt>,
        total_dmg -> Nullable<BigInt>,
        total_heal -> Nullable<BigInt>,
    }
}

diesel::table! {
    damage_events (id) {
        id -> Integer,
        encounter_id -> Integer,
        timestamp_ms -> BigInt,
        attacker_id -> BigInt,
        defender_id -> Nullable<BigInt>,
        monster_name -> Nullable<Text>,
        skill_id -> Nullable<Integer>,
        value -> BigInt,
        is_crit -> Integer,
        is_lucky -> Integer,
        hp_loss -> BigInt,
        shield_loss -> BigInt,
        is_boss -> Integer,
    }
}

diesel::table! {
    heal_events (id) {
        id -> Integer,
        encounter_id -> Integer,
        timestamp_ms -> BigInt,
        healer_id -> BigInt,
        target_id -> Nullable<BigInt>,
        skill_id -> Nullable<Integer>,
        value -> BigInt,
        is_crit -> Integer,
        is_lucky -> Integer,
    }
}

diesel::table! {
    actor_encounter_stats (encounter_id, actor_id) {
        encounter_id -> Integer,
        actor_id -> BigInt,
        damage_dealt -> BigInt,
        heal_dealt -> BigInt,
        damage_taken -> BigInt,
        hits_dealt -> BigInt,
        hits_heal -> BigInt,
        hits_taken -> BigInt,
        crit_hits_dealt -> BigInt,
        crit_hits_heal -> BigInt,
        crit_hits_taken -> BigInt,
        lucky_hits_dealt -> BigInt,
        lucky_hits_heal -> BigInt,
        lucky_hits_taken -> BigInt,
        crit_total_dealt -> BigInt,
        crit_total_heal -> BigInt,
        crit_total_taken -> BigInt,
        lucky_total_dealt -> BigInt,
        lucky_total_heal -> BigInt,
        lucky_total_taken -> BigInt,
        boss_damage_dealt -> BigInt,
        boss_hits_dealt -> BigInt,
        boss_crit_hits_dealt -> BigInt,
        boss_lucky_hits_dealt -> BigInt,
        boss_crit_total_dealt -> BigInt,
        boss_lucky_total_dealt -> BigInt,
    }
}

// Joins

diesel::joinable!(damage_events -> encounters (encounter_id));
diesel::joinable!(heal_events -> encounters (encounter_id));

diesel::allow_tables_to_appear_in_same_query!(
    entities,
    skills,
    encounters,
    damage_events,
    heal_events,
    actor_encounter_stats,
);
