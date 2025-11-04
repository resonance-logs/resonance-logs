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
        attributes -> Nullable<Text>,
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
        scene_id -> Nullable<Integer>,
        scene_name -> Nullable<Text>,
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
        defender_max_hp -> Nullable<BigInt>,
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
        name -> Nullable<Text>,
        class_id -> Nullable<Integer>,
        class_spec -> Nullable<Integer>,
        ability_score -> Nullable<Integer>,
        level -> Nullable<Integer>,
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
        is_player -> Integer,
        is_local_player -> Integer,
    }
}

diesel::table! {
    damage_skill_stats (encounter_id, attacker_id, defender_id, skill_id) {
        encounter_id -> Integer,
        attacker_id -> BigInt,
        defender_id -> Nullable<BigInt>,
        skill_id -> Integer,
        hits -> Integer,
        total_value -> BigInt,
        crit_hits -> Integer,
        lucky_hits -> Integer,
        crit_total -> BigInt,
        lucky_total -> BigInt,
        hp_loss_total -> BigInt,
        shield_loss_total -> BigInt,
        hit_details -> Text,
        monster_name -> Nullable<Text>,
    }
}

diesel::table! {
    heal_skill_stats (encounter_id, healer_id, target_id, skill_id) {
        encounter_id -> Integer,
        healer_id -> BigInt,
        target_id -> Nullable<BigInt>,
        skill_id -> Integer,
        hits -> Integer,
        total_value -> BigInt,
        crit_hits -> Integer,
        lucky_hits -> Integer,
        crit_total -> BigInt,
        lucky_total -> BigInt,
        heal_details -> Text,
        monster_name -> Nullable<Text>,
    }
}

diesel::table! {
    encounter_bosses (encounter_id, monster_name) {
        encounter_id -> Integer,
        monster_name -> Text,
        hits -> Integer,
        total_damage -> BigInt,
        max_hp -> Nullable<BigInt>,
        is_defeated -> Integer,
    }
}

// Joins

diesel::joinable!(damage_events -> encounters (encounter_id));
diesel::joinable!(heal_events -> encounters (encounter_id));
diesel::joinable!(damage_skill_stats -> encounters (encounter_id));
diesel::joinable!(heal_skill_stats -> encounters (encounter_id));
diesel::joinable!(encounter_bosses -> encounters (encounter_id));

diesel::allow_tables_to_appear_in_same_query!(
    entities,
    encounters,
    damage_events,
    heal_events,
    actor_encounter_stats,
    damage_skill_stats,
    heal_skill_stats,
    encounter_bosses,
);
