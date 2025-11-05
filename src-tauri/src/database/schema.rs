// @generated automatically by Diesel CLI, but maintained manually here.
// Keep in sync with migrations.

/// Represents the `entities` table.
diesel::table! {
    entities (entity_id) {
        /// The unique ID of the entity.
        entity_id -> BigInt,
        /// The name of the entity.
        name -> Nullable<Text>,
        /// The class ID of the entity.
        class_id -> Nullable<Integer>,
        /// The class spec of the entity.
        class_spec -> Nullable<Integer>,
        /// The ability score of the entity.
        ability_score -> Nullable<Integer>,
        /// The level of the entity.
        level -> Nullable<Integer>,
        /// The timestamp of when the entity was first seen, in milliseconds since the Unix epoch.
        first_seen_ms -> Nullable<BigInt>,
        /// The timestamp of when the entity was last seen, in milliseconds since the Unix epoch.
        last_seen_ms -> Nullable<BigInt>,
        /// The attributes of the entity.
        attributes -> Nullable<Text>,
    }
}

/// Represents the `encounters` table.
diesel::table! {
    encounters (id) {
        /// The unique ID of the encounter.
        id -> Integer,
        /// The timestamp of when the encounter started, in milliseconds since the Unix epoch.
        started_at_ms -> BigInt,
        /// The timestamp of when the encounter ended, in milliseconds since the Unix epoch.
        ended_at_ms -> Nullable<BigInt>,
        /// The ID of the local player.
        local_player_id -> Nullable<BigInt>,
        /// The total damage dealt in the encounter.
        total_dmg -> Nullable<BigInt>,
        /// The total healing done in the encounter.
        total_heal -> Nullable<BigInt>,
        /// The ID of the scene where the encounter took place.
        scene_id -> Nullable<Integer>,
        /// The name of the scene where the encounter took place.
        scene_name -> Nullable<Text>,
    }
}

/// Represents the `damage_events` table.
diesel::table! {
    damage_events (id) {
        /// The unique ID of the damage event.
        id -> Integer,
        /// The ID of the encounter this event belongs to.
        encounter_id -> Integer,
        /// The timestamp of the event, in milliseconds since the Unix epoch.
        timestamp_ms -> BigInt,
        /// The ID of the attacker.
        attacker_id -> BigInt,
        /// The ID of the defender.
        defender_id -> Nullable<BigInt>,
        /// The name of the monster.
        monster_name -> Nullable<Text>,
        /// The ID of the skill used.
        skill_id -> Nullable<Integer>,
        /// The value of the damage.
        value -> BigInt,
        /// Whether the damage was a critical hit.
        is_crit -> Integer,
        /// Whether the damage was a lucky hit.
        is_lucky -> Integer,
        /// The amount of HP lost.
        hp_loss -> BigInt,
        /// The amount of shield lost.
        shield_loss -> BigInt,
        /// The maximum HP of the defender.
        defender_max_hp -> Nullable<BigInt>,
        /// Whether the target was a boss.
        is_boss -> Integer,
    }
}

/// Represents the `heal_events` table.
diesel::table! {
    heal_events (id) {
        /// The unique ID of the heal event.
        id -> Integer,
        /// The ID of the encounter this event belongs to.
        encounter_id -> Integer,
        /// The timestamp of the event, in milliseconds since the Unix epoch.
        timestamp_ms -> BigInt,
        /// The ID of the healer.
        healer_id -> BigInt,
        /// The ID of the target.
        target_id -> Nullable<BigInt>,
        /// The ID of the skill used.
        skill_id -> Nullable<Integer>,
        /// The value of the heal.
        value -> BigInt,
        /// Whether the heal was a critical hit.
        is_crit -> Integer,
        /// Whether the heal was a lucky hit.
        is_lucky -> Integer,
    }
}

/// Represents the `actor_encounter_stats` table.
diesel::table! {
    actor_encounter_stats (encounter_id, actor_id) {
        /// The ID of the encounter.
        encounter_id -> Integer,
        /// The ID of the actor.
        actor_id -> BigInt,
        /// The name of the actor.
        name -> Nullable<Text>,
        /// The class ID of the actor.
        class_id -> Nullable<Integer>,
        /// The class spec of the actor.
        class_spec -> Nullable<Integer>,
        /// The ability score of the actor.
        ability_score -> Nullable<Integer>,
        /// The level of the actor.
        level -> Nullable<Integer>,
        /// The total damage dealt by the actor.
        damage_dealt -> BigInt,
        /// The total healing done by the actor.
        heal_dealt -> BigInt,
        /// The total damage taken by the actor.
        damage_taken -> BigInt,
        /// The number of hits dealt by the actor.
        hits_dealt -> BigInt,
        /// The number of hits healed by the actor.
        hits_heal -> BigInt,
        /// The number of hits taken by the actor.
        hits_taken -> BigInt,
        /// The number of critical hits dealt by the actor.
        crit_hits_dealt -> BigInt,
        /// The number of critical hits healed by the actor.
        crit_hits_heal -> BigInt,
        /// The number of critical hits taken by the actor.
        crit_hits_taken -> BigInt,
        /// The number of lucky hits dealt by the actor.
        lucky_hits_dealt -> BigInt,
        /// The number of lucky hits healed by the actor.
        lucky_hits_heal -> BigInt,
        /// The number of lucky hits taken by the actor.
        lucky_hits_taken -> BigInt,
        /// The total critical damage dealt by the actor.
        crit_total_dealt -> BigInt,
        /// The total critical healing done by the actor.
        crit_total_heal -> BigInt,
        /// The total critical damage taken by the actor.
        crit_total_taken -> BigInt,
        /// The total lucky damage dealt by the actor.
        lucky_total_dealt -> BigInt,
        /// The total lucky healing done by the actor.
        lucky_total_heal -> BigInt,
        /// The total lucky damage taken by the actor.
        lucky_total_taken -> BigInt,
        /// The total damage dealt to bosses by the actor.
        boss_damage_dealt -> BigInt,
        /// The number of hits dealt to bosses by the actor.
        boss_hits_dealt -> BigInt,
        /// The number of critical hits dealt to bosses by the actor.
        boss_crit_hits_dealt -> BigInt,
        /// The number of lucky hits dealt to bosses by the actor.
        boss_lucky_hits_dealt -> BigInt,
        /// The total critical damage dealt to bosses by the actor.
        boss_crit_total_dealt -> BigInt,
        /// The total lucky damage dealt to bosses by the actor.
        boss_lucky_total_dealt -> BigInt,
        /// Whether the actor is a player.
        is_player -> Integer,
        /// Whether the actor is the local player.
        is_local_player -> Integer,
    }
}

/// Represents the `damage_skill_stats` table.
diesel::table! {
    damage_skill_stats (encounter_id, attacker_id, defender_id, skill_id) {
        /// The ID of the encounter.
        encounter_id -> Integer,
        /// The ID of the attacker.
        attacker_id -> BigInt,
        /// The ID of the defender.
        defender_id -> Nullable<BigInt>,
        /// The ID of the skill used.
        skill_id -> Integer,
        /// The number of hits.
        hits -> Integer,
        /// The total value of the damage.
        total_value -> BigInt,
        /// The number of critical hits.
        crit_hits -> Integer,
        /// The number of lucky hits.
        lucky_hits -> Integer,
        /// The total critical damage.
        crit_total -> BigInt,
        /// The total lucky damage.
        lucky_total -> BigInt,
        /// The total HP lost.
        hp_loss_total -> BigInt,
        /// The total shield lost.
        shield_loss_total -> BigInt,
        /// The details of the hits.
        hit_details -> Text,
        /// The name of the monster.
        monster_name -> Nullable<Text>,
    }
}

/// Represents the `heal_skill_stats` table.
diesel::table! {
    heal_skill_stats (encounter_id, healer_id, target_id, skill_id) {
        /// The ID of the encounter.
        encounter_id -> Integer,
        /// The ID of the healer.
        healer_id -> BigInt,
        /// The ID of the target.
        target_id -> Nullable<BigInt>,
        /// The ID of the skill used.
        skill_id -> Integer,
        /// The number of hits.
        hits -> Integer,
        /// The total value of the heal.
        total_value -> BigInt,
        /// The number of critical hits.
        crit_hits -> Integer,
        /// The number of lucky hits.
        lucky_hits -> Integer,
        /// The total critical heal.
        crit_total -> BigInt,
        /// The total lucky heal.
        lucky_total -> BigInt,
        /// The details of the heals.
        heal_details -> Text,
        /// The name of the monster.
        monster_name -> Nullable<Text>,
    }
}

/// Represents the `encounter_bosses` table.
diesel::table! {
    encounter_bosses (encounter_id, monster_name) {
        /// The ID of the encounter.
        encounter_id -> Integer,
        /// The name of the monster.
        monster_name -> Text,
        /// The number of hits.
        hits -> Integer,
        /// The total damage dealt to the boss.
        total_damage -> BigInt,
        /// The maximum HP of the boss.
        max_hp -> Nullable<BigInt>,
        /// Whether the boss was defeated.
        is_defeated -> Integer,
    }
}

/// Represents the `death_events` table.
diesel::table! {
    death_events (id) {
        /// The unique ID of the death event.
        id -> Integer,
        /// The ID of the encounter this event belongs to.
        encounter_id -> Integer,
        /// The timestamp of the event, in milliseconds since the Unix epoch.
        timestamp_ms -> BigInt,
        /// The ID of the actor who died.
        actor_id -> BigInt,
        /// The ID of the killer (if known).
        killer_id -> Nullable<BigInt>,
        /// The skill ID that caused the death (if known).
        skill_id -> Nullable<Integer>,
        /// Whether the actor was the local player.
        is_local_player -> Integer,
    }
}

// Joins

diesel::joinable!(damage_events -> encounters (encounter_id));
diesel::joinable!(heal_events -> encounters (encounter_id));
diesel::joinable!(damage_skill_stats -> encounters (encounter_id));
diesel::joinable!(heal_skill_stats -> encounters (encounter_id));
diesel::joinable!(encounter_bosses -> encounters (encounter_id));
diesel::joinable!(death_events -> encounters (encounter_id));

diesel::allow_tables_to_appear_in_same_query!(
    entities,
    encounters,
    damage_events,
    heal_events,
    actor_encounter_stats,
    damage_skill_stats,
    heal_skill_stats,
    encounter_bosses,
    death_events,
);
