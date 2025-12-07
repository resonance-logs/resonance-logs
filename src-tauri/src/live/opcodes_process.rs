// NOTE: opcodes_process works on Encounter directly; avoid importing opcodes_models at top-level.
use crate::database::{DbTask, enqueue, now_ms};
use crate::live::attempt_detector::{
    AttemptConfig, check_hp_rollback_condition, check_wipe_condition, get_boss_hp_percentage,
    split_attempt, track_party_member, update_boss_hp_tracking,
};
use crate::live::dungeon_log::{self, DungeonLogRuntime};
use crate::live::opcodes_models::class::{
    ClassSpec, get_class_id_from_spec, get_class_spec_from_skill_id,
};
use crate::live::opcodes_models::{
    AttrType, AttrValue, BuffEvent, Encounter, Entity, Skill, attr_type,
};
use crate::packets::utils::BinaryReader;
use blueprotobuf_lib::blueprotobuf;
use blueprotobuf_lib::blueprotobuf::{Attr, EDamageType, EEntityType};
use log::info;
use std::collections::HashMap;
use std::default::Default;
use std::time::{SystemTime, UNIX_EPOCH};

/// Record a death event into the encounter and enqueue a DB task.
fn record_death(
    encounter: &mut Encounter,
    actor_id: i64,
    killer_id: Option<i64>,
    skill_id: Option<i32>,
    timestamp_ms: i64,
) {
    // Dedupe close-together events for the same actor (2s window) using a
    // dedicated map for DB death inserts. We no longer use death tracking for
    // wipe detection/UI; death events are still persisted to the DB.
    let should_record = match encounter.last_death_db_ms.get(&actor_id) {
        Some(last_ms) => {
            let diff = (timestamp_ms as i128 - *last_ms as i128).abs();
            diff > 2000
        }
        None => true,
    };

    if !should_record {
        return;
    }

    encounter
        .last_death_db_ms
        .insert(actor_id, timestamp_ms as u128);

    // Enqueue DB task; mark as local player when matching tracked local UID
    let is_local = encounter.local_player_uid == actor_id;
    enqueue(DbTask::InsertDeathEvent {
        timestamp_ms,
        actor_id,
        killer_id,
        skill_id,
        is_local_player: is_local,
        attempt_index: Some(encounter.current_attempt_index),
    });
}

/// Record a revive event into the encounter for UI emission.
fn record_revive(encounter: &mut Encounter, actor_id: i64, timestamp_ms: i64) {
    // Dedupe close-together revives for the same actor (2s window)
    let should_record = match encounter.last_revive_ms.get(&actor_id) {
        Some(last_ms) => {
            let diff = (timestamp_ms as i128 - *last_ms as i128).abs();
            diff > 2000
        }
        None => true,
    };

    if !should_record {
        return;
    }

    encounter
        .last_revive_ms
        .insert(actor_id, timestamp_ms as u128);

    // Push to pending player revives for UI emission
    encounter
        .pending_player_revives
        .push((actor_id, None, None, timestamp_ms));

    info!("Recorded revive for UID {}", actor_id);
}

fn did_target_die(
    is_dead_flag: Option<bool>,
    hp_loss: u128,
    shield_loss: u128,
    prev_hp: Option<i64>,
    max_hp: Option<i64>,
) -> bool {
    if let Some(true) = is_dead_flag {
        return true;
    }

    let total_loss = hp_loss.saturating_add(shield_loss);
    if total_loss == 0 {
        return false;
    }

    if let Some(prev_hp_val) = prev_hp.filter(|hp| *hp > 0) {
        let prev_hp_u128 = prev_hp_val as u128;
        if total_loss >= prev_hp_u128 {
            return true;
        }
    }

    if let Some(max_hp_val) = max_hp.filter(|hp| *hp > 0) {
        let max_hp_u128 = max_hp_val as u128;
        if total_loss >= max_hp_u128 {
            return true;
        }
    }

    false
}

/// Serialize entity attributes HashMap to JSON string for database storage.
/// Converts AttrType keys to string representation for JSON compatibility.
fn serialize_attributes(entity: &Entity) -> Option<String> {
    if entity.attributes.is_empty() {
        return None;
    }

    // Convert HashMap<AttrType, AttrValue> to HashMap<String, serde_json::Value> for JSON serialization
    // This is necessary because JSON object keys must be strings, and AttrType::Unknown(i32)
    // cannot be directly serialized as a JSON object key
    use crate::live::opcodes_models::{AttrType, AttrValue};
    use serde_json::json;

    let string_map: serde_json::Map<String, serde_json::Value> = entity
        .attributes
        .iter()
        .map(|(k, v)| {
            let key_str = match k {
                AttrType::Unknown(id) => format!("Unknown_0x{:x}", id),
                _ => format!("{:?}", k), // Uses Debug trait for named variants
            };
            let value_json = match v {
                AttrValue::Int(i) => json!(i),
                AttrValue::Float(f) => json!(f),
                AttrValue::String(s) => json!(s),
                AttrValue::Bool(b) => json!(b),
            };
            (key_str, value_json)
        })
        .collect();

    serde_json::to_string(&string_map).ok()
}

pub fn on_server_change(encounter: &mut Encounter) {
    info!("on server change");
    // Preserve entity identity and local player info; only reset combat state
    encounter.reset_combat_state();
}

/// Process a NotifyReviveUser packet: record a revive for the actor.
///
/// This will add a revive entry to the encounter's pending revives for UI emission
/// (we no longer clear death markers here because death tracking is not used for
/// wipe detection).
pub fn process_notify_revive_user(
    encounter: &mut Encounter,
    notify_revive: blueprotobuf::NotifyReviveUser,
) -> Option<()> {
    let actor_uuid = notify_revive.v_actor_uuid?;
    // Actor UUID in protobuf is signed i64; interpret bits as u64 for shifting
    let actor_uuid_u = actor_uuid as u64;
    let uid = (actor_uuid_u >> 16) as i64;

    // Record revive for UI emission (timestamp using now_ms helper)
    let ts = now_ms();
    record_revive(encounter, uid, ts);
    // Persist revive to DB (increment per-actor revive counter)
    let is_local = encounter.local_player_uid == uid;
    enqueue(DbTask::InsertReviveEvent {
        timestamp_ms: ts,
        actor_id: uid,
        is_local_player: is_local,
        attempt_index: Some(encounter.current_attempt_index),
    });
    info!(
        "Processed NotifyReviveUser: recorded revive for UID {}",
        uid
    );
    Some(())
}

pub fn process_sync_near_entities(
    encounter: &mut Encounter,
    sync_near_entities: blueprotobuf::SyncNearEntities,
) -> Option<()> {
    for pkt_entity in sync_near_entities.appear {
        let target_uuid = pkt_entity.uuid?;
        let target_uid = target_uuid >> 16;
        let target_entity_type = EEntityType::from(target_uuid);

        let target_entity = encounter
            .entity_uid_to_entity
            .entry(target_uid)
            .or_default();
        target_entity.entity_type = target_entity_type;

        match target_entity_type {
            EEntityType::EntChar => {
                process_player_attrs(target_entity, target_uid, pkt_entity.attrs?.attrs);
            }
            EEntityType::EntMonster => {
                process_monster_attrs(target_entity, pkt_entity.attrs?.attrs);
            }
            _ => {}
        }

        // Lazy upsert entity into DB (only players are persisted)
        if matches!(target_entity_type, EEntityType::EntChar) {
            let name_opt = if target_entity.name.is_empty() {
                None
            } else {
                Some(target_entity.name.clone())
            };
            enqueue(DbTask::UpsertEntity {
                entity_id: target_uid,
                name: name_opt,
                class_id: Some(target_entity.class_id),
                class_spec: Some(target_entity.class_spec as i32),
                ability_score: Some(target_entity.ability_score),
                level: Some(target_entity.level),
                seen_at_ms: now_ms(),
                attributes: serialize_attributes(target_entity),
            });
        }
    }

    // Track party members for wipe detection (collect data first to avoid borrow issues)
    let player_data: Vec<(i64, EEntityType, Option<i64>)> = encounter
        .entity_uid_to_entity
        .iter()
        .filter_map(|(uid, entity)| {
            if entity.entity_type == EEntityType::EntChar {
                Some((*uid, entity.entity_type, entity.team_id()))
            } else {
                None
            }
        })
        .collect();

    for (uid, entity_type, team_id) in player_data {
        track_party_member(encounter, uid, entity_type, team_id);
    }

    Some(())
}

pub fn process_sync_container_data(
    encounter: &mut Encounter,
    sync_container_data: blueprotobuf::SyncContainerData,
) -> Option<()> {
    use crate::live::opcodes_models::{AttrType, AttrValue};

    let v_data = sync_container_data.v_data?;
    let player_uid = v_data.char_id?;

    let target_entity = encounter
        .entity_uid_to_entity
        .entry(player_uid)
        .or_default();
    let char_base = v_data.char_base.as_ref()?;
    let name = char_base.name.clone()?;
    target_entity.name = name;
    target_entity.set_attr(
        AttrType::Name,
        AttrValue::String(target_entity.name.clone()),
    );

    // Player names are automatically stored in the database via UpsertEntity tasks
    // No need to maintain a separate cache anymore
    target_entity.entity_type = EEntityType::EntChar;
    let profession_list = v_data.profession_list.as_ref()?;
    let class_id = profession_list.cur_profession_id?;
    target_entity.class_id = class_id;
    target_entity.set_attr(
        AttrType::ProfessionId,
        AttrValue::Int(target_entity.class_id as i64),
    );

    target_entity.ability_score = char_base.fight_point?;
    target_entity.set_attr(
        AttrType::FightPoint,
        AttrValue::Int(target_entity.ability_score as i64),
    );

    let role_level = v_data.role_level.as_ref()?;
    target_entity.level = role_level.level?;
    target_entity.set_attr(AttrType::Level, AttrValue::Int(target_entity.level as i64));

    // Note: HP data comes from attribute packets (ATTR_CURRENT_HP, ATTR_MAX_HP)
    // CharBaseInfo doesn't contain HP fields

    // Lazy upsert with richer info
    let name_opt = if target_entity.name.is_empty() {
        None
    } else {
        Some(target_entity.name.clone())
    };
    // Only store players in the database
    if matches!(target_entity.entity_type, EEntityType::EntChar) {
        enqueue(DbTask::UpsertEntity {
            entity_id: player_uid,
            name: name_opt,
            class_id: Some(target_entity.class_id),
            class_spec: Some(target_entity.class_spec as i32),
            ability_score: Some(target_entity.ability_score),
            level: Some(target_entity.level),
            seen_at_ms: now_ms(),
            attributes: serialize_attributes(target_entity),
        });

        // Persist detailed player data for the local player.
        let now = now_ms();

        // Serialize the full CharSerialize payload (including nested structures).
        let char_serialize_json = serde_json::to_string(&v_data).unwrap_or_else(|e| {
            log::warn!("Failed to serialize CharSerialize payload: {}", e);
            "{}".to_string()
        });

        // Extract profession_list for easier access / smaller payloads downstream.
        let profession_list_json = serde_json::to_string(profession_list).ok();

        // Extract talent node ids from each ProfessionTalentInfo entry.
        let talent_node_ids_json = {
            let talent_map: std::collections::HashMap<i32, Vec<u32>> = profession_list
                .talent_list
                .iter()
                .map(|(profession_id, talent_info)| {
                    (*profession_id, talent_info.talent_node_ids.clone())
                })
                .collect();
            serde_json::to_string(&talent_map).ok()
        };

        enqueue(DbTask::UpsertDetailedPlayerData {
            player_id: player_uid,
            last_seen_ms: now,
            char_serialize_json,
            profession_list_json,
            talent_node_ids_json,
        });
    }

    Some(())
}

pub fn process_sync_container_dirty_data(
    _encounter: &mut Encounter,
    _sync_container_dirty_data: blueprotobuf::SyncContainerDirtyData,
) -> Option<()> {
    // SyncContainerDirtyData.v_data is a BufferStream (raw bytes)
    // Incremental attribute updates come through process_player_attrs via AoiSyncDelta
    // which handles attr packets with proper typing
    Some(())
}

pub fn process_sync_to_me_delta_info(
    encounter: &mut Encounter,
    sync_to_me_delta_info: blueprotobuf::SyncToMeDeltaInfo,
    dungeon_runtime: Option<&DungeonLogRuntime>,
    config: &AttemptConfig,
) -> Option<()> {
    let delta_info = match sync_to_me_delta_info.delta_info {
        Some(info) => info,
        None => {
            // This is normal during gameplay - packet may not always contain delta_info
            return None;
        }
    };

    if let Some(uuid) = delta_info.uuid {
        encounter.local_player_uid = uuid >> 16; // UUID =/= uid (have to >> 16)
    }

    if let Some(base_delta) = delta_info.base_delta {
        process_aoi_sync_delta(encounter, base_delta, dungeon_runtime, config);
    }

    Some(())
}

pub fn process_aoi_sync_delta(
    encounter: &mut Encounter,
    aoi_sync_delta: blueprotobuf::AoiSyncDelta,
    dungeon_runtime: Option<&DungeonLogRuntime>,
    config: &AttemptConfig,
) -> Option<()> {
    let target_uuid = aoi_sync_delta.uuid?; // UUID =/= uid (have to >> 16)
    let target_uid = target_uuid >> 16;

    // Process attributes
    let target_entity_type = EEntityType::from(target_uuid);
    let mut target_entity = encounter
        .entity_uid_to_entity
        .entry(target_uid)
        .or_insert_with(|| Entity {
            entity_type: target_entity_type,
            ..Default::default()
        });

    if let Some(attrs_collection) = aoi_sync_delta.attrs {
        match target_entity_type {
            EEntityType::EntChar => {
                process_player_attrs(&mut target_entity, target_uid, attrs_collection.attrs);
            }
            EEntityType::EntMonster => {
                process_monster_attrs(&mut target_entity, attrs_collection.attrs);
            }
            _ => {}
        }

        // Lazy upsert target entity after attrs
        let name_opt = if target_entity.name.is_empty() {
            None
        } else {
            Some(target_entity.name.clone())
        };
        // Only store players in the database
        if matches!(target_entity_type, EEntityType::EntChar) {
            enqueue(DbTask::UpsertEntity {
                entity_id: target_uid,
                name: name_opt,
                class_id: Some(target_entity.class_id),
                class_spec: Some(target_entity.class_spec as i32),
                ability_score: Some(target_entity.ability_score),
                level: Some(target_entity.level),
                seen_at_ms: now_ms(),
                attributes: serialize_attributes(target_entity),
            });
        }
    }

    // Process BuffInfoSync if present - track buff applications
    if let Some(ref buff_info_sync) = aoi_sync_delta.buff_infos {
        let fight_start = encounter.time_fight_start_ms as i64;
        let now = now_ms();
        for buff_info in &buff_info_sync.buff_infos {
            // Extract buff data
            let buff_id = match buff_info.base_id {
                Some(id) => id,
                None => continue,
            };
            let duration = buff_info.duration.unwrap_or(0);
            let stack_count = buff_info.layer.unwrap_or(1);
            let create_time = buff_info.create_time.unwrap_or(now);

            // Calculate start time relative to fight start (or use absolute if no fight)
            let start = if fight_start > 0 {
                (create_time - fight_start).max(0)
            } else {
                create_time
            };
            let end = start + (duration as i64);

            let key = (target_uid, buff_id);
            let events = encounter.buff_events.entry(key).or_insert_with(Vec::new);

            // Try to find an existing event that this update belongs to:
            // 1. Same start time = stack update on existing buff instance
            // 2. Start is within an existing event's window = refresh that extends duration
            let mut found_match = false;
            for existing in events.iter_mut() {
                if existing.start == start && existing.duration == duration {
                    // Same buff instance, just update stack count to latest value
                    existing.stack_count = stack_count;
                    found_match = true;
                    break;
                } else if start >= existing.start && start < existing.end {
                    // Buff was refreshed mid-duration - extend the end time
                    existing.end = end;
                    existing.stack_count = stack_count;
                    found_match = true;
                    break;
                }
            }

            // Only add as new event if no matching existing event found
            if !found_match {
                events.push(BuffEvent {
                    start,
                    end,
                    duration,
                    stack_count,
                });
            }
        }
    }

    let Some(skill_effect) = aoi_sync_delta.skill_effects else {
        return Some(()); // return ok since this variable usually doesn't exist
    };

    // Process Damage
    for sync_damage_info in skill_effect.damages {
        // Timestamp for this event
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let timestamp_ms_i64 = timestamp_ms.min(i64::MAX as u128) as i64;
        let non_lucky_dmg = sync_damage_info.value;
        let lucky_value = sync_damage_info.lucky_value;

        #[allow(clippy::cast_sign_loss)]
        let actual_value = if let Some(actual_dmg) = non_lucky_dmg.or(lucky_value) {
            actual_dmg as u128
        } else {
            continue; // skip this iteration
        };

        let attacker_uuid = sync_damage_info
            .top_summoner_id
            .or(sync_damage_info.attacker_uuid)?;
        let attacker_uid = attacker_uuid >> 16;

        // Local copies of fields needed later (avoid holding map borrows across operations)
        let skill_uid = sync_damage_info.owner_id?;
        let flag = sync_damage_info.type_flag.unwrap_or_default();
        // Pre-calculate whether this target is recognized as a boss and local player id
        let is_boss_target = encounter
            .entity_uid_to_entity
            .get(&target_uid)
            .map(|e| e.is_boss())
            .unwrap_or(false);
        let local_player_uid_copy = encounter.local_player_uid;

        // First update attacker-side state in its own scope (single mutable borrow)
        let (is_crit, is_lucky, attacker_entity_type_copy, was_heal_event) = {
            let attacker_entity = encounter
                .entity_uid_to_entity
                .entry(attacker_uid)
                .or_insert_with(|| Entity {
                    entity_type: EEntityType::from(attacker_uuid),
                    ..Default::default()
                });

            if attacker_entity.class_spec == ClassSpec::Unknown {
                let class_spec = get_class_spec_from_skill_id(skill_uid);
                attacker_entity.class_id = get_class_id_from_spec(class_spec);
                attacker_entity.class_spec = class_spec;
            }

            let is_heal = sync_damage_info.r#type.unwrap_or(0) == EDamageType::Heal as i32;
            let is_lucky_local = lucky_value.is_some();
            const CRIT_BIT: i32 = 0b00_00_00_01;
            let is_crit_local = (flag & CRIT_BIT) != 0;

            if is_heal {
                let skill = attacker_entity
                    .skill_uid_to_heal_skill
                    .entry(skill_uid)
                    .or_insert_with(|| Skill::default());
                if is_crit_local {
                    attacker_entity.crit_hits_heal += 1;
                    attacker_entity.crit_total_heal += actual_value;
                    skill.crit_hits += 1;
                    skill.crit_total_value += actual_value;
                }
                if is_lucky_local {
                    attacker_entity.lucky_hits_heal += 1;
                    attacker_entity.lucky_total_heal += actual_value;
                    skill.lucky_hits += 1;
                    skill.lucky_total_value += actual_value;
                }
                encounter.total_heal += actual_value;
                attacker_entity.hits_heal += 1;
                attacker_entity.total_heal += actual_value;
                skill.hits += 1;
                skill.total_value += actual_value;

                // Persist attacker
                if matches!(attacker_entity.entity_type, EEntityType::EntChar) {
                    enqueue(DbTask::UpsertEntity {
                        entity_id: attacker_uid,
                        name: if attacker_entity.name.is_empty() {
                            None
                        } else {
                            Some(attacker_entity.name.clone())
                        },
                        class_id: Some(attacker_entity.class_id),
                        class_spec: Some(attacker_entity.class_spec as i32),
                        ability_score: Some(attacker_entity.ability_score),
                        level: Some(attacker_entity.level),
                        seen_at_ms: timestamp_ms_i64,
                        attributes: serialize_attributes(attacker_entity),
                    });
                }

                // Insert heal event
                enqueue(DbTask::InsertHealEvent {
                    timestamp_ms: timestamp_ms_i64,
                    healer_id: attacker_uid,
                    target_id: Some(target_uid),
                    skill_id: Some(skill_uid),
                    value: actual_value as i64,
                    is_crit: is_crit_local,
                    is_lucky: is_lucky_local,
                    attempt_index: Some(encounter.current_attempt_index),
                });

                (
                    is_crit_local,
                    is_lucky_local,
                    attacker_entity.entity_type,
                    true,
                )
            } else {
                let skill = attacker_entity
                    .skill_uid_to_dmg_skill
                    .entry(skill_uid)
                    .or_insert_with(|| Skill::default());
                if is_crit_local {
                    attacker_entity.crit_hits_dmg += 1;
                    attacker_entity.crit_total_dmg += actual_value;
                    skill.crit_hits += 1;
                    skill.crit_total_value += actual_value;
                }
                if is_lucky_local {
                    attacker_entity.lucky_hits_dmg += 1;
                    attacker_entity.lucky_total_dmg += actual_value;
                    skill.lucky_hits += 1;
                    skill.lucky_total_value += actual_value;
                }
                encounter.total_dmg += actual_value;
                attacker_entity.hits_dmg += 1;
                attacker_entity.total_dmg += actual_value;
                skill.hits += 1;
                skill.total_value += actual_value;

                if is_boss_target {
                    let skill_boss_only = attacker_entity
                        .skill_uid_to_dmg_skill_boss_only
                        .entry(skill_uid)
                        .or_insert_with(|| Skill::default());
                    if is_crit_local {
                        attacker_entity.crit_hits_dmg_boss_only += 1;
                        attacker_entity.crit_total_dmg_boss_only += actual_value;
                        skill_boss_only.crit_hits += 1;
                        skill_boss_only.crit_total_value += actual_value;
                    }
                    if is_lucky_local {
                        attacker_entity.lucky_hits_dmg_boss_only += 1;
                        attacker_entity.lucky_total_dmg_boss_only += actual_value;
                        skill_boss_only.lucky_hits += 1;
                        skill_boss_only.lucky_total_value += actual_value;
                    }
                    encounter.total_dmg_boss_only += actual_value;
                    attacker_entity.hits_dmg_boss_only += 1;
                    attacker_entity.total_dmg_boss_only += actual_value;
                    skill_boss_only.hits += 1;
                    skill_boss_only.total_value += actual_value;
                }

                // Track per-target totals
                use std::collections::hash_map::Entry;
                match attacker_entity.dmg_to_target.entry(target_uid) {
                    Entry::Occupied(mut e) => {
                        *e.get_mut() += actual_value;
                    }
                    Entry::Vacant(e) => {
                        e.insert(actual_value);
                    }
                }
                let per_skill = attacker_entity
                    .skill_dmg_to_target
                    .entry(skill_uid)
                    .or_insert_with(HashMap::new);
                match per_skill.entry(target_uid) {
                    Entry::Occupied(mut e) => {
                        *e.get_mut() += actual_value;
                    }
                    Entry::Vacant(e) => {
                        e.insert(actual_value);
                    }
                }

                // Persist attacker
                if matches!(attacker_entity.entity_type, EEntityType::EntChar) {
                    enqueue(DbTask::UpsertEntity {
                        entity_id: attacker_uid,
                        name: if attacker_entity.name.is_empty() {
                            None
                        } else {
                            Some(attacker_entity.name.clone())
                        },
                        class_id: Some(attacker_entity.class_id),
                        class_spec: Some(attacker_entity.class_spec as i32),
                        ability_score: Some(attacker_entity.ability_score),
                        level: Some(attacker_entity.level),
                        seen_at_ms: timestamp_ms_i64,
                        attributes: serialize_attributes(attacker_entity),
                    });
                }

                (
                    is_crit_local,
                    is_lucky_local,
                    attacker_entity.entity_type,
                    false,
                )
            }
        };

        // Now handle defender-side updates in their own scope and compute death info
        let (death_info_local, target_name, target_monster_type_id) = {
            // Track damage taken
            let hp_loss = sync_damage_info.hp_lessen_value.unwrap_or(0).max(0) as u128;
            let shield_loss = sync_damage_info.shield_lessen_value.unwrap_or(0).max(0) as u128;
            let effective_value = if hp_loss + shield_loss > 0 {
                hp_loss + shield_loss
            } else {
                actual_value
            };

            let defender_entity = encounter
                .entity_uid_to_entity
                .entry(target_uid)
                .or_insert_with(|| Entity {
                    entity_type: EEntityType::from(target_uuid),
                    ..Default::default()
                });

            let target_name = if defender_entity.name.is_empty() {
                None
            } else {
                Some(defender_entity.name.clone())
            };
            let target_monster_type_id = defender_entity.monster_type_id.map(|id| i64::from(id));

            // Check for death
            let prev_hp_opt = defender_entity.hp();
            let max_hp_opt = defender_entity.max_hp();
            let died = did_target_die(
                sync_damage_info.is_dead,
                hp_loss,
                shield_loss,
                prev_hp_opt,
                max_hp_opt,
            );

            // Persist defender
            if matches!(defender_entity.entity_type, EEntityType::EntChar) {
                enqueue(DbTask::UpsertEntity {
                    entity_id: target_uid,
                    name: if defender_entity.name.is_empty() {
                        None
                    } else {
                        Some(defender_entity.name.clone())
                    },
                    class_id: Some(defender_entity.class_id),
                    class_spec: Some(defender_entity.class_spec as i32),
                    ability_score: Some(defender_entity.ability_score),
                    level: Some(defender_entity.level),
                    seen_at_ms: timestamp_ms_i64,
                    attributes: serialize_attributes(defender_entity),
                });
            }

            // Only record damage/taken stats if this event is not a heal
            if !was_heal_event {
                // Insert damage event
                let is_boss = defender_entity.is_boss();
                let monster_name_for_event =
                    if matches!(defender_entity.entity_type, EEntityType::EntMonster) {
                        defender_entity.monster_name_packet.clone().or_else(|| {
                            if defender_entity.name.is_empty() {
                                None
                            } else {
                                Some(defender_entity.name.clone())
                            }
                        })
                    } else {
                        None
                    };
                enqueue(DbTask::InsertDamageEvent {
                    timestamp_ms: timestamp_ms_i64,
                    attacker_id: attacker_uid,
                    defender_id: Some(target_uid),
                    monster_name: monster_name_for_event,
                    skill_id: Some(skill_uid),
                    value: effective_value as i64,
                    is_crit,
                    is_lucky,
                    hp_loss: hp_loss as i64,
                    shield_loss: shield_loss as i64,
                    defender_max_hp: defender_entity
                        .attributes
                        .get(&AttrType::MaxHp)
                        .and_then(|v| v.as_int()),
                    is_boss,
                    attempt_index: Some(encounter.current_attempt_index),
                });

                // Taken stats (only when attacker is not a player)
                if attacker_entity_type_copy != EEntityType::EntChar {
                    let taken_skill = defender_entity
                        .skill_uid_to_taken_skill
                        .entry(skill_uid)
                        .or_insert_with(|| Skill::default());
                    if is_crit {
                        defender_entity.crit_hits_taken += 1;
                        defender_entity.crit_total_taken += effective_value;
                        taken_skill.crit_hits += 1;
                        taken_skill.crit_total_value += effective_value;
                    }
                    if is_lucky {
                        defender_entity.lucky_hits_taken += 1;
                        defender_entity.lucky_total_taken += effective_value;
                        taken_skill.lucky_hits += 1;
                        taken_skill.lucky_total_value += effective_value;
                    }
                    defender_entity.hits_taken += 1;
                    defender_entity.total_taken += effective_value;
                    taken_skill.hits += 1;
                    taken_skill.total_value += effective_value;
                }
            }

            let death_info = if died {
                Some((
                    target_uid,
                    Some(attacker_uid),
                    Some(skill_uid),
                    timestamp_ms_i64,
                ))
            } else {
                None
            };

            (death_info, target_name, target_monster_type_id)
        };

        if let Some(runtime) = dungeon_runtime {
            if !was_heal_event {
                let damage_amount = actual_value.min(i64::MAX as u128) as i64;
                let is_boss_target_hint = encounter
                    .entity_uid_to_entity
                    .get(&target_uid)
                    .map(|entity| entity.is_boss())
                    .unwrap_or(false);

                let damage_event = dungeon_log::build_damage_event(
                    timestamp_ms_i64,
                    attacker_uid,
                    target_uid,
                    target_name.clone(),
                    target_monster_type_id,
                    damage_amount,
                    death_info_local.is_some(),
                    is_boss_target_hint,
                );
                let (boss_died, new_boss_started) = runtime.process_damage_event(damage_event);

                // Persist segments if a boss died or a new boss started (implies previous segment closed)
                if boss_died || new_boss_started {
                    dungeon_log::persist_segments(&runtime.shared_log, false);
                }

                // Check for segment type transitions and reset metrics if needed
                // Prioritize open boss segments over trash - during a boss fight with adds,
                // we don't want to switch to "trash" just because trash mob damage occurred.
                let current_segment_type =
                    dungeon_log::snapshot(&runtime.shared_log).and_then(|log| {
                        // First check for any open boss segment
                        let open_boss = log.segments.iter().rev().find(|s| {
                            s.segment_type == dungeon_log::SegmentType::Boss
                                && s.ended_at_ms.is_none()
                        });
                        if open_boss.is_some() {
                            return Some("boss".to_string());
                        }
                        // Fall back to any open segment (trash)
                        log.segments
                            .iter()
                            .rev()
                            .find(|s| s.ended_at_ms.is_none())
                            .map(|s| match s.segment_type {
                                dungeon_log::SegmentType::Boss => "boss".to_string(),
                                dungeon_log::SegmentType::Trash => "trash".to_string(),
                            })
                    });

                // If segment type changed, reset the live meter
                if let Some(current_type) = &current_segment_type {
                    if encounter.last_active_segment_type.as_ref() != Some(current_type) {
                        info!(
                            "Segment type changed from {:?} to {}, resetting live meter",
                            encounter.last_active_segment_type, current_type
                        );

                        // Reset only player metrics, preserving boss HP attributes
                        // so the boss health bar remains visible during segment switches
                        encounter.reset_segment_metrics();

                        // Update the last segment type
                        encounter.last_active_segment_type = Some(current_type.clone());
                    }
                }

                // If a boss just died, set the waiting flag
                if boss_died {
                    encounter.waiting_for_next_boss = true;
                }

                // If a new boss started while we were waiting, clear the waiting flag
                if new_boss_started && encounter.waiting_for_next_boss {
                    encounter.waiting_for_next_boss = false;
                }
            }
        }

        // If death detected, record it (dedupe handled inside record_death)
        if let Some((actor, killer, skill, ts)) = death_info_local {
            record_death(encounter, actor, killer, skill, ts);
        }
    }

    // Figure out timestamps (moved earlier for use in attempt detection)
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    // Check for wipe condition after recording deaths
    if check_wipe_condition(encounter, config) {
        let boss_hp = encounter
            .entity_uid_to_entity
            .values()
            .find(|e| e.is_boss())
            .and_then(|e| e.hp());
        split_attempt(encounter, "wipe", timestamp_ms, boss_hp);
    }

    // Check for HP rollback after processing damage events
    if let Some(boss_hp_pct) = get_boss_hp_percentage(encounter) {
        // Update boss HP tracking - find boss HP first
        let boss_hp_opt = encounter
            .entity_uid_to_entity
            .values()
            .find(|e| e.is_boss())
            .and_then(|e| e.hp());

        if let Some(boss_hp) = boss_hp_opt {
            update_boss_hp_tracking(encounter, boss_hp);
        }

        // Check for HP rollback
        if check_hp_rollback_condition(encounter, Some(boss_hp_pct), config) {
            let boss_hp = encounter
                .entity_uid_to_entity
                .values()
                .find(|e| e.is_boss())
                .and_then(|e| e.hp());
            split_attempt(encounter, "hp_rollback", timestamp_ms, boss_hp);
        }
    }

    if encounter.time_fight_start_ms == Default::default() {
        encounter.time_fight_start_ms = timestamp_ms;

        // Only persist encounters to the database for non-overworld scenes.
        // Scene ID 8 is the overworld; we still track and display the encounter
        // in-memory, but avoid creating DB rows for overworld so the DB stays
        // free of non-dungeon activity.
        let persist_to_db = match encounter.current_scene_id {
            Some(id) if id == 8 => false,
            _ => true,
        };

        if persist_to_db {
            enqueue(DbTask::BeginEncounter {
                started_at_ms: timestamp_ms as i64,
                local_player_id: Some(encounter.local_player_uid),
                scene_id: encounter.current_scene_id,
                scene_name: encounter.current_scene_name.clone(),
            });

            // Determine current boss HP (if a boss entity is present) and begin first attempt
            let initial_boss_hp = encounter
                .entity_uid_to_entity
                .values()
                .find(|e| e.is_boss())
                .and_then(|e| e.hp());

            // Begin first attempt with boss HP if available
            enqueue(DbTask::BeginAttempt {
                attempt_index: 1,
                started_at_ms: timestamp_ms as i64,
                reason: "initial".to_string(),
                boss_hp_start: initial_boss_hp,
            });

            // Initialize encounter tracking for attempts
            encounter.boss_hp_at_attempt_start = initial_boss_hp;
            if let Some(bhp) = initial_boss_hp {
                // Initialize lowest boss HP percentage tracking
                update_boss_hp_tracking(encounter, bhp);
            }
        } else {
            // When not persisting to DB (overworld), still initialize attempt tracking
            // in-memory so the live meter shows correct data. We do NOT enqueue any
            // DB tasks in this branch.
            let initial_boss_hp = encounter
                .entity_uid_to_entity
                .values()
                .find(|e| e.is_boss())
                .and_then(|e| e.hp());
            encounter.boss_hp_at_attempt_start = initial_boss_hp;
            if let Some(bhp) = initial_boss_hp {
                update_boss_hp_tracking(encounter, bhp);
            }
        }
    }

    encounter.time_last_combat_packet_ms = timestamp_ms;
    Some(())
}

fn process_player_attrs(player_entity: &mut Entity, target_uid: i64, attrs: Vec<Attr>) {
    use crate::live::opcodes_models::{AttrType, AttrValue};
    use bytes::Buf;

    for attr in attrs {
        let Some(raw_bytes) = attr.raw_data else {
            continue;
        };
        let Some(attr_id) = attr.id else { continue };

        // Create a bytes buffer for protobuf decoding
        let mut buf = &raw_bytes[..];

        match attr_id {
            attr_type::ATTR_NAME => {
                // Decode protobuf string (varint length prefix + UTF-8 bytes)
                match prost::encoding::decode_varint(&mut buf) {
                    Ok(len) => {
                        let len = len as usize;
                        if buf.remaining() >= len {
                            let bytes = buf.copy_to_bytes(len);
                            match String::from_utf8(bytes.to_vec()) {
                                Ok(player_name) => {
                                    player_entity.name = player_name.clone();
                                    player_entity.set_attr(
                                        AttrType::Name,
                                        AttrValue::String(player_name.clone()),
                                    );
                                    info! {"Found player {} with UID {}", player_entity.name, target_uid}

                                    // Store player in database
                                    if matches!(player_entity.entity_type, EEntityType::EntChar) {
                                        enqueue(DbTask::UpsertEntity {
                                            entity_id: target_uid,
                                            name: Some(player_name),
                                            class_id: Some(player_entity.class_id),
                                            class_spec: Some(player_entity.class_spec as i32),
                                            ability_score: Some(player_entity.ability_score),
                                            level: Some(player_entity.level),
                                            seen_at_ms: now_ms(),
                                            attributes: serialize_attributes(player_entity),
                                        });
                                    }
                                }
                                Err(e) => log::warn!(
                                    "Failed to decode ATTR_NAME UTF-8 for UID {}: {:?}",
                                    target_uid,
                                    e
                                ),
                            }
                        } else {
                            log::warn!("ATTR_NAME buffer too short for UID {}", target_uid);
                        }
                    }
                    Err(e) => log::warn!(
                        "Failed to decode ATTR_NAME varint for UID {}: {:?}",
                        target_uid,
                        e
                    ),
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_PROFESSION_ID => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    let value = value as i32;
                    player_entity.class_id = value;
                    player_entity.set_attr(AttrType::ProfessionId, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_PROFESSION_ID: {:?}", e),
            },
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_FIGHT_POINT => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    let value = value as i32;
                    player_entity.ability_score = value;
                    player_entity.set_attr(AttrType::FightPoint, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_FIGHT_POINT: {:?}", e),
            },
            #[allow(clippy::cast_possible_truncation)]
            attr_type::ATTR_LEVEL => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    let value = value as i32;
                    player_entity.level = value;
                    player_entity.set_attr(AttrType::Level, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_LEVEL: {:?}", e),
            },
            attr_type::ATTR_RANK_LEVEL => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::RankLevel, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_RANK_LEVEL: {:?}", e),
            },
            attr_type::ATTR_CRIT => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::Crit, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_CRIT: {:?}", e),
            },
            attr_type::ATTR_LUCKY => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::Lucky, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_LUCKY: {:?}", e),
            },
            attr_type::ATTR_CURRENT_HP => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::CurrentHp, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_CURRENT_HP: {:?}", e),
            },
            attr_type::ATTR_MAX_HP => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::MaxHp, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MAX_HP: {:?}", e),
            },
            attr_type::ATTR_HASTE => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::Haste, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_HASTE: {:?}", e),
            },
            attr_type::ATTR_MASTERY => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::Mastery, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MASTERY: {:?}", e),
            },
            attr_type::ATTR_ELEMENT_FLAG => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::ElementFlag, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_ELEMENT_FLAG: {:?}", e),
            },
            attr_type::ATTR_ENERGY_FLAG => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::EnergyFlag, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_ENERGY_FLAG: {:?}", e),
            },
            attr_type::ATTR_REDUCTION_LEVEL => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::ReductionLevel, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_REDUCTION_LEVEL: {:?}", e),
            },
            attr_type::ATTR_TEAM_ID => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::TeamId, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_TEAM_ID: {:?}", e),
            },
            attr_type::ATTR_ATTACK_POWER => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::AttackPower, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_ATTACK_POWER: {:?}", e),
            },
            attr_type::ATTR_DEFENSE_POWER => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::DefensePower, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_DEFENSE_POWER: {:?}", e),
            },
            attr_type::ATTR_STAR_LEVEL => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::StarLevel, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_STAR_LEVEL: {:?}", e),
            },
            attr_type::ATTR_GEAR_TIER => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::GearTier, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_GEAR_TIER: {:?}", e),
            },
            attr_type::ATTR_PVP_RANK => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::PvpRank, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_PVP_RANK: {:?}", e),
            },
            attr_type::ATTR_TOTAL_POWER => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::TotalPower, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_TOTAL_POWER: {:?}", e),
            },
            attr_type::ATTR_PHYSICAL_ATTACK => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::PhysicalAttack, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_PHYSICAL_ATTACK: {:?}", e),
            },
            attr_type::ATTR_MAGIC_ATTACK => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::MagicAttack, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MAGIC_ATTACK: {:?}", e),
            },
            attr_type::ATTR_WEAPON_TYPE => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::WeaponType, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_WEAPON_TYPE: {:?}", e),
            },
            attr_type::ATTR_RESURRECTION_COUNT => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity
                        .set_attr(AttrType::ResurrectionCount, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_RESURRECTION_COUNT: {:?}", e),
            },
            attr_type::ATTR_PARTY_ROLE => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::PartyRole, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_PARTY_ROLE: {:?}", e),
            },
            attr_type::ATTR_COMBAT_STATE => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::CombatState, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_COMBAT_STATE: {:?}", e),
            },
            attr_type::ATTR_EQUIPMENT_SLOT_1 => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::EquipmentSlot1, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_EQUIPMENT_SLOT_1: {:?}", e),
            },
            attr_type::ATTR_EQUIPMENT_SLOT_2 => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::EquipmentSlot2, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_EQUIPMENT_SLOT_2: {:?}", e),
            },
            attr_type::ATTR_CURRENT_SHIELD => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::CurrentShield, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_CURRENT_SHIELD: {:?}", e),
            },
            attr_type::ATTR_ELEMENTAL_RES_1 => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::ElementalRes1, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_ELEMENTAL_RES_1: {:?}", e),
            },
            attr_type::ATTR_ELEMENTAL_RES_2 => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::ElementalRes2, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_ELEMENTAL_RES_2: {:?}", e),
            },
            attr_type::ATTR_ELEMENTAL_RES_3 => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::ElementalRes3, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_ELEMENTAL_RES_3: {:?}", e),
            },
            attr_type::ATTR_BUFF_SLOT => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::BuffSlot, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_BUFF_SLOT: {:?}", e),
            },
            attr_type::ATTR_GUILD_ID => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::GuildId, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_GUILD_ID: {:?}", e),
            },
            attr_type::ATTR_GENDER => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::Gender, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_GENDER: {:?}", e),
            },
            attr_type::ATTR_TOTAL_DEFENSE => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::TotalDefense, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_TOTAL_DEFENSE: {:?}", e),
            },
            attr_type::ATTR_ENDURANCE => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::Endurance, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_ENDURANCE: {:?}", e),
            },
            attr_type::ATTR_CHARACTER_TIMESTAMP => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity
                        .set_attr(AttrType::CharacterTimestamp, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_CHARACTER_TIMESTAMP: {:?}", e),
            },
            attr_type::ATTR_SESSION_TIMESTAMP => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity
                        .set_attr(AttrType::SessionTimestamp, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_SESSION_TIMESTAMP: {:?}", e),
            },
            attr_type::ATTR_MOVEMENT_SPEED => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::MovementSpeed, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MOVEMENT_SPEED: {:?}", e),
            },
            attr_type::ATTR_TALENT_SPEC => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::TalentSpec, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_TALENT_SPEC: {:?}", e),
            },
            attr_type::ATTR_ELITE_STATUS => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::EliteStatus, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_ELITE_STATUS: {:?}", e),
            },
            attr_type::ATTR_MAX_MP => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::MaxMp, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MAX_MP: {:?}", e),
            },
            attr_type::ATTR_STAMINA => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::Stamina, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_STAMINA: {:?}", e),
            },
            attr_type::ATTR_BUFF_SLOT_2 => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::BuffSlot2, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_BUFF_SLOT_2: {:?}", e),
            },
            attr_type::ATTR_BASE_STRENGTH => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::BaseStrength, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_BASE_STRENGTH: {:?}", e),
            },
            attr_type::ATTR_COMBAT_MODE => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::CombatMode, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_COMBAT_MODE: {:?}", e),
            },
            attr_type::ATTR_LAST_ACTION_TIMESTAMP => {
                match prost::encoding::decode_varint(&mut buf) {
                    Ok(value) => {
                        player_entity
                            .set_attr(AttrType::LastActionTimestamp, AttrValue::Int(value as i64));
                    }
                    Err(e) => log::warn!("Failed to decode ATTR_LAST_ACTION_TIMESTAMP: {:?}", e),
                }
            }
            attr_type::ATTR_BUFF_SLOT_3 => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::BuffSlot3, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_BUFF_SLOT_3: {:?}", e),
            },
            attr_type::ATTR_MOUNT_STATUS => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::MountStatus, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MOUNT_STATUS: {:?}", e),
            },
            attr_type::ATTR_MOUNT_TIMESTAMP => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::MountTimestamp, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MOUNT_TIMESTAMP: {:?}", e),
            },
            attr_type::ATTR_MOUNT_SPEED => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::MountSpeed, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MOUNT_SPEED: {:?}", e),
            },
            attr_type::ATTR_MOUNT_DURATION => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::MountDuration, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MOUNT_DURATION: {:?}", e),
            },
            attr_type::ATTR_MIN_ENERGY => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::MinEnergy, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MIN_ENERGY: {:?}", e),
            },
            attr_type::ATTR_MAX_ENERGY => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::MaxEnergy, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MAX_ENERGY: {:?}", e),
            },
            attr_type::ATTR_ENERGY_REGEN => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity.set_attr(AttrType::EnergyRegen, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_ENERGY_REGEN: {:?}", e),
            },
            attr_type::ATTR_PHYSICAL_PENETRATION => {
                match prost::encoding::decode_varint(&mut buf) {
                    Ok(value) => {
                        player_entity
                            .set_attr(AttrType::PhysicalPenetration, AttrValue::Int(value as i64));
                    }
                    Err(e) => log::warn!("Failed to decode ATTR_PHYSICAL_PENETRATION: {:?}", e),
                }
            }
            attr_type::ATTR_MAGIC_PENETRATION => match prost::encoding::decode_varint(&mut buf) {
                Ok(value) => {
                    player_entity
                        .set_attr(AttrType::MagicPenetration, AttrValue::Int(value as i64));
                }
                Err(e) => log::warn!("Failed to decode ATTR_MAGIC_PENETRATION: {:?}", e),
            },
            _ => {
                // Store unknown attribute IDs with their decoded values
                // This captures all attributes, even ones we don't explicitly handle yet
                if attr_id > 0
                    && !matches!(attr_id, attr_type::ATTR_ID | attr_type::ATTR_REDUCTION_ID)
                {
                    use crate::live::opcodes_models::AttrValue;

                    // Try to decode as varint first (most common)
                    let mut debug_buf = &raw_bytes[..];
                    match prost::encoding::decode_varint(&mut debug_buf) {
                        Ok(val) => {
                            // Store as unknown varint attribute
                            player_entity
                                .set_attr(AttrType::Unknown(attr_id), AttrValue::Int(val as i64));
                            // log::trace!("Unknown player attribute ID: 0x{:x} = {}", attr_id, val);
                        }
                        Err(_) => {
                            // Try as string
                            let mut str_buf = &raw_bytes[..];
                            match prost::encoding::decode_varint(&mut str_buf) {
                                Ok(len) => {
                                    if str_buf.remaining() >= len as usize {
                                        let bytes = str_buf.copy_to_bytes(len as usize);
                                        match String::from_utf8(bytes.to_vec()) {
                                            Ok(s) => {
                                                // Store as unknown string attribute
                                                player_entity.set_attr(
                                                    AttrType::Unknown(attr_id),
                                                    AttrValue::String(s.clone()),
                                                );
                                                // log::trace!(
                                                //     "Unknown player attribute ID: 0x{:x} = \"{}\"",
                                                //     attr_id,
                                                //     s
                                                // );
                                            }
                                            Err(_) => {
                                                // Store as hex string for binary data
                                                let hex_str: String = raw_bytes
                                                    .iter()
                                                    .map(|b| format!("{:02x}", b))
                                                    .collect::<Vec<_>>()
                                                    .join("");
                                                player_entity.set_attr(
                                                    AttrType::Unknown(attr_id),
                                                    AttrValue::String(format!("0x{}", hex_str)),
                                                );
                                                // log::trace!(
                                                //     "Unknown player attribute ID: 0x{:x} = hex {}",
                                                //     attr_id,
                                                //     hex_str
                                                // );
                                            }
                                        }
                                    } else {
                                        let hex_str: String = raw_bytes
                                            .iter()
                                            .map(|b| format!("{:02x}", b))
                                            .collect::<Vec<_>>()
                                            .join("");
                                        player_entity.set_attr(
                                            AttrType::Unknown(attr_id),
                                            AttrValue::String(format!("0x{}", hex_str)),
                                        );
                                        // log::trace!(
                                        //     "Unknown player attribute ID: 0x{:x} = hex {}",
                                        //     attr_id,
                                        //     hex_str
                                        // );
                                    }
                                }
                                Err(_) => {
                                    let hex_str: String = raw_bytes
                                        .iter()
                                        .map(|b| format!("{:02x}", b))
                                        .collect::<Vec<_>>()
                                        .join("");
                                    player_entity.set_attr(
                                        AttrType::Unknown(attr_id),
                                        AttrValue::String(format!("0x{}", hex_str)),
                                    );
                                    // log::trace!(
                                    //     "Unknown player attribute ID: 0x{:x} = hex {}",
                                    //     attr_id,
                                    //     hex_str
                                    // );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn process_monster_attrs(monster_entity: &mut Entity, attrs: Vec<Attr>) {
    use crate::live::opcodes_models::attr_type;
    for attr in attrs {
        let Some(mut raw_bytes) = attr.raw_data else {
            continue;
        };
        let Some(attr_id) = attr.id else { continue };
        match attr_id {
            attr_type::ATTR_ID => {
                let monster_id =
                    prost::encoding::decode_varint(&mut raw_bytes.as_slice()).unwrap_or(0) as i32;
                if monster_id > 0 {
                    monster_entity.set_monster_type(monster_id);
                }
            }
            attr_type::ATTR_NAME => {
                if !raw_bytes.is_empty() {
                    raw_bytes.remove(0);
                }
                if let Ok(name) = BinaryReader::from(raw_bytes).read_string() {
                    // Always capture the raw packet name for monsters
                    monster_entity.monster_name_packet = Some(name.clone());
                    if monster_entity.monster_type_id.is_none() {
                        monster_entity.name = name;
                    }
                }
            }
            attr_type::ATTR_CURRENT_HP => {
                if let Ok(value) = prost::encoding::decode_varint(&mut raw_bytes.as_slice()) {
                    monster_entity.set_attr(AttrType::CurrentHp, AttrValue::Int(value as i64));
                }
            }
            attr_type::ATTR_MAX_HP => {
                if let Ok(value) = prost::encoding::decode_varint(&mut raw_bytes.as_slice()) {
                    monster_entity.set_attr(AttrType::MaxHp, AttrValue::Int(value as i64));
                }
            }
            attr_type::ATTR_ELITE_STATUS => {
                match prost::encoding::decode_varint(&mut raw_bytes.as_slice()) {
                    Ok(value) => {
                        monster_entity
                            .set_attr(AttrType::EliteStatus, AttrValue::Int(value as i64));
                    }
                    Err(e) => log::warn!("Failed to decode ATTR_ELITE_STATUS: {:?}", e),
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::did_target_die;

    #[test]
    fn uses_packet_flag_when_present() {
        assert!(did_target_die(Some(true), 0, 0, None, None));
        assert!(!did_target_die(Some(false), 0, 0, Some(10), Some(20)));
    }

    #[test]
    fn hp_loss_must_exceed_previous_hp() {
        assert!(!did_target_die(None, 50, 0, Some(100), Some(200)));
        assert!(did_target_die(None, 150, 0, Some(100), Some(200)));
    }

    #[test]
    fn falls_back_to_max_hp_when_needed() {
        assert!(did_target_die(None, 1_500, 0, None, Some(1_000)));
        assert!(!did_target_die(None, 500, 0, None, Some(1_000)));
    }

    #[test]
    fn zero_loss_never_marks_death() {
        assert!(!did_target_die(None, 0, 0, Some(1), Some(2)));
    }
}
