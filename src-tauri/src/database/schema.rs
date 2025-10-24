// @generated automatically by Diesel CLI, but maintained manually here.
// Keep in sync with migrations.

diesel::table! {
    sessions (id) {
        id -> Integer,
        started_at_ms -> BigInt,
        ended_at_ms -> Nullable<BigInt>,
        version -> Nullable<Text>,
        platform -> Nullable<Text>,
    }
}

diesel::table! {
    entities (entity_id) {
        entity_id -> BigInt,
        entity_type -> Integer,
        is_player -> Integer,
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
        session_id -> Nullable<Integer>,
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
        skill_id -> Nullable<Integer>,
        value -> BigInt,
        is_crit -> Integer,
        is_lucky -> Integer,
        hp_loss -> BigInt,
        shield_loss -> BigInt,
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

// Joins

diesel::joinable!(damage_events -> encounters (encounter_id));
diesel::joinable!(heal_events -> encounters (encounter_id));

diesel::allow_tables_to_appear_in_same_query!(
    sessions,
    entities,
    skills,
    encounters,
    damage_events,
    heal_events,
);
