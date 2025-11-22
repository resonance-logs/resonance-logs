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

diesel::table! {
    detailed_playerdata (player_id) {
        player_id -> BigInt,
        last_seen_ms -> BigInt,
        char_serialize_json -> Text,
        profession_list_json -> Nullable<Text>,
        talent_node_ids_json -> Nullable<Text>,
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
        /// The duration of the encounter in seconds.
        duration -> Double,
        /// Timestamp (ms) when this encounter was successfully uploaded to the website.
        uploaded_at_ms -> Nullable<BigInt>,
        /// The encounter ID on the remote website/server after successful upload.
        remote_encounter_id -> Nullable<BigInt>,
    }
}

/// Represents the `damage_events` table.
// Raw per-event tables `damage_events` and `heal_events` have been removed.
// The codebase now stores only aggregated `_stats` tables (damage_skill_stats, heal_skill_stats,
// actor_encounter_stats, encounter_bosses, etc.). The Diesel table macros for raw event
// tables were intentionally removed as part of the irreversible schema change.

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
        /// The number of revives for the actor during the encounter.
        revives -> BigInt,
        /// The average DPS snapshot for the actor during the encounter.
        dps -> Double,
        /// The encounter duration in seconds used for the DPS snapshot.
        duration -> Double,
        /// Whether the actor is a player.
        is_player -> Integer,
        /// Whether the actor is the local player.
        is_local_player -> Integer,
        /// The attributes of the actor.
        attributes -> Nullable<Text>,
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
        /// The attempt index this death occurred in.
        attempt_index -> Nullable<Integer>,
    }
}

/// Represents the `attempts` table.
diesel::table! {
    attempts (id) {
        /// The unique ID of the attempt.
        id -> Integer,
        /// The ID of the encounter this attempt belongs to.
        encounter_id -> Integer,
        /// The attempt index (1-based).
        attempt_index -> Integer,
        /// The timestamp of when the attempt started, in milliseconds since the Unix epoch.
        started_at_ms -> BigInt,
        /// The timestamp of when the attempt ended, in milliseconds since the Unix epoch.
        ended_at_ms -> Nullable<BigInt>,
        /// The reason for the attempt split ('wipe', 'hp_rollback', 'manual').
        reason -> Text,
        /// The boss HP at the start of the attempt.
        boss_hp_start -> Nullable<BigInt>,
        /// The boss HP at the end of the attempt.
        boss_hp_end -> Nullable<BigInt>,
        /// The total number of deaths in this attempt.
        total_deaths -> Integer,
    }
}

/// Represents the `encounter_phases` table.
diesel::table! {
    encounter_phases (id) {
        /// The unique ID of the encounter phase.
        id -> Integer,
        /// The ID of the encounter this phase belongs to.
        encounter_id -> Integer,
        /// The type of phase ('mob' or 'boss').
        phase_type -> Text,
        /// The timestamp of when the phase started, in milliseconds since the Unix epoch.
        start_time_ms -> BigInt,
        /// The timestamp of when the phase ended, in milliseconds since the Unix epoch.
        end_time_ms -> Nullable<BigInt>,
        /// The outcome of the phase ('success', 'wipe', 'unknown').
        outcome -> Text,
    }
}

/// Represents the `actor_phase_stats` table.
diesel::table! {
    actor_phase_stats (phase_id, actor_id) {
        /// The ID of the phase.
        phase_id -> Integer,
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
        /// The number of revives for the actor during the phase.
        revives -> BigInt,
        /// Whether the actor is a player.
        is_player -> Integer,
        /// Whether the actor is the local player.
        is_local_player -> Integer,
        /// The attributes of the actor.
        attributes -> Nullable<Text>,
    }
}

/// Represents the `dungeon_segments` table.
diesel::table! {
    dungeon_segments (id) {
        /// The unique ID of the segment.
        id -> Integer,
        /// The ID of the encounter this segment belongs to.
        encounter_id -> Integer,
        /// The type of segment ('boss' or 'trash').
        segment_type -> Text,
        /// The entity ID of the boss (if boss segment).
        boss_entity_id -> Nullable<BigInt>,
        /// The monster type ID of the boss (if boss segment).
        boss_monster_type_id -> Nullable<BigInt>,
        /// The name of the boss (if boss segment).
        boss_name -> Nullable<Text>,
        /// The timestamp of when the segment started, in milliseconds since the Unix epoch.
        started_at_ms -> BigInt,
        /// The timestamp of when the segment ended, in milliseconds since the Unix epoch.
        ended_at_ms -> Nullable<BigInt>,
        /// The total damage dealt during this segment.
        total_damage -> BigInt,
        /// The number of hits during this segment.
        hit_count -> BigInt,
    }
}

// Joins

// joinable entries for raw event tables removed
diesel::joinable!(damage_skill_stats -> encounters (encounter_id));
diesel::joinable!(heal_skill_stats -> encounters (encounter_id));
diesel::joinable!(encounter_bosses -> encounters (encounter_id));
diesel::joinable!(death_events -> encounters (encounter_id));
diesel::joinable!(attempts -> encounters (encounter_id));
diesel::joinable!(encounter_phases -> encounters (encounter_id));
diesel::joinable!(actor_phase_stats -> encounter_phases (phase_id));
diesel::joinable!(dungeon_segments -> encounters (encounter_id));

diesel::allow_tables_to_appear_in_same_query!(
    entities,
    encounters,
    actor_encounter_stats,
    detailed_playerdata,
    damage_skill_stats,
    heal_skill_stats,
    encounter_bosses,
    death_events,
    attempts,
    encounter_phases,
    actor_phase_stats,
    dungeon_segments,
);
