use crate::live::event_manager::{
    generate_header_info, generate_players_window_dps, generate_players_window_heal,
    generate_skills_window_dps, generate_skills_window_heal, EventManagerMutex, MetricType,
};
use crate::live::opcodes_models::EncounterMutex;
use crate::live::skills_store::SkillsStoreMutex;
use crate::live::opcodes_process::{
    on_server_change, process_aoi_sync_delta, process_sync_container_data,
    process_sync_container_dirty_data, process_sync_near_entities, process_sync_to_me_delta_info,
};
use crate::packets;
use blueprotobuf_lib::blueprotobuf;
use bytes::Bytes;
use log::{info, trace, warn};
use prost::Message;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

pub async fn start(app_handle: AppHandle) {
    // todo: add app_handle?
    // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
    // 1. Start capturing packets and send to rx
    let mut rx = packets::packet_capture::start_capture(); // Since live meter is not critical, it's ok to just log it // TODO: maybe bubble an error up to the frontend instead?

    // Initialize event manager
    {
        let event_manager_state = app_handle.state::<EventManagerMutex>();
        let mut event_manager = event_manager_state.lock().unwrap();
        event_manager.initialize(app_handle.clone());
    }

    // Throttling for events (emit at most every 100ms)
    let mut last_emit_time = Instant::now();
    let emit_throttle_duration = Duration::from_millis(100);

    // Track previous state to detect changes
    let mut last_header_info: Option<crate::live::commands_models::HeaderInfo> = None;
    let mut last_is_paused: Option<bool> = None;

    // 2. Use the channel to receive packets back and process them
    while let Some((op, data)) = rx.recv().await {
        {
            let state = app_handle.state::<EncounterMutex>();
            let encounter = state.lock().unwrap();
            if encounter.is_encounter_paused {
                info!("packet dropped due to encounter paused");
                continue;
            }
        }
        // error!("Received Pkt {op:?}");
        match op {
            packets::opcodes::Pkt::ServerChangeInfo => {
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                on_server_change(&mut encounter_state);

                // Reset cached state
                last_header_info = None;
                last_is_paused = None;

                // Emit encounter reset event
                let event_manager_state = app_handle.state::<EventManagerMutex>();
                let event_manager = event_manager_state.lock().unwrap();
                if event_manager.should_emit_events() {
                    event_manager.emit_encounter_reset();
                }

                // Clear skills store and subscriptions
                let skills_store_state = app_handle.state::<SkillsStoreMutex>();
                let mut skills_store = skills_store_state.lock().unwrap();
                skills_store.clear();
            }
            packets::opcodes::Pkt::SyncNearEntities => {
                // info!("Received {op:?}");
                // info!("Received {op:?} and data {data:?}");
                // trace!("Received {op:?} and data {data:?}");
                let sync_near_entities =
                    match blueprotobuf::SyncNearEntities::decode(Bytes::from(data)) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Error decoding SyncNearEntities.. ignoring: {e}");
                            continue;
                        }
                    };
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                if process_sync_near_entities(&mut encounter_state, sync_near_entities).is_none() {
                    warn!("Error processing SyncNearEntities.. ignoring.");
                }
            }
            packets::opcodes::Pkt::SyncContainerData => {
                // info!("Received {op:?}");
                // info!("Received {op:?} and data {data:?}");
                // trace!("Received {op:?} and data {data:?}");
                let sync_container_data =
                    match blueprotobuf::SyncContainerData::decode(Bytes::from(data)) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Error decoding SyncContainerData.. ignoring: {e}");
                            continue;
                        }
                    };
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                encounter_state.local_player = sync_container_data.clone();
                if process_sync_container_data(&mut encounter_state, sync_container_data).is_none()
                {
                    warn!("Error processing SyncContainerData.. ignoring.");
                }
            }
            packets::opcodes::Pkt::SyncContainerDirtyData => {
                // info!("Received {op:?}");
                // trace!("Received {op:?} and data {data:?}");
                let sync_container_dirty_data =
                    match blueprotobuf::SyncContainerDirtyData::decode(Bytes::from(data)) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Error decoding SyncContainerDirtyData.. ignoring: {e}");
                            continue;
                        }
                    };
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                if process_sync_container_dirty_data(
                    &mut encounter_state,
                    sync_container_dirty_data,
                )
                .is_none()
                {
                    warn!("Error processing SyncToMeDeltaInfo.. ignoring.");
                }
            }
            packets::opcodes::Pkt::SyncServerTime => {
                // info!("Received {op:?}");
                // trace!("Received {op:?} and data {data:?}");
                let _sync_server_time =
                    match blueprotobuf::SyncServerTime::decode(Bytes::from(data)) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Error decoding SyncServerTime.. ignoring: {e}");
                            continue;
                        }
                    };
                // todo: this is skipped, not sure what info it has
            }
            packets::opcodes::Pkt::SyncToMeDeltaInfo => {
                // todo: fix this, attrs dont include name, no idea why
                // trace!("Received {op:?}");
                // info!("Received {op:?} and data {data:?}");
                let sync_to_me_delta_info =
                    match blueprotobuf::SyncToMeDeltaInfo::decode(Bytes::from(data)) {
                        Ok(sync_to_me_delta_info) => sync_to_me_delta_info,
                        Err(e) => {
                            warn!("Error decoding SyncToMeDeltaInfo.. ignoring: {e}");
                            continue;
                        }
                    };
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                if process_sync_to_me_delta_info(&mut encounter_state, sync_to_me_delta_info)
                    .is_none()
                {
                    warn!("Error processing SyncToMeDeltaInfo.. ignoring.");
                }
            }
            packets::opcodes::Pkt::SyncNearDeltaInfo => {
                // trace!("Received {op:?}");
                // info!("Received {op:?} and data {data:?}");
                let sync_near_delta_info =
                    match blueprotobuf::SyncNearDeltaInfo::decode(Bytes::from(data)) {
                        Ok(v) => v,
                        Err(e) => {
                            warn!("Error decoding SyncNearDeltaInfo.. ignoring: {e}");
                            continue;
                        }
                    };
                let encounter_state = app_handle.state::<EncounterMutex>();
                let mut encounter_state = encounter_state.lock().unwrap();
                for aoi_sync_delta in sync_near_delta_info.delta_infos {
                    if process_aoi_sync_delta(&mut encounter_state, aoi_sync_delta).is_none() {
                        warn!("Error processing SyncToMeDeltaInfo.. ignoring.");
                    }
                }

                // Check if we should emit events (throttling)
                let now = Instant::now();
                if now.duration_since(last_emit_time) >= emit_throttle_duration {
                    last_emit_time = now;

                    // Emit events for updated data
                    let event_manager_state = app_handle.state::<EventManagerMutex>();
                    let event_manager = event_manager_state.lock().unwrap();
                    if event_manager.should_emit_events() {
                        // Generate and emit encounter update only if it changed
                        if let Some(header_info) = generate_header_info(&encounter_state) {
                            let is_paused = encounter_state.is_encounter_paused;
                            let should_emit = last_header_info.as_ref() != Some(&header_info)
                                || last_is_paused != Some(is_paused);

                            if should_emit {
                                event_manager.emit_encounter_update(header_info.clone(), is_paused);
                                last_header_info = Some(header_info);
                                last_is_paused = Some(is_paused);
                            }
                        }

                        // Generate and emit DPS players update only if there's data
                        let dps_players = generate_players_window_dps(&encounter_state);
                        if !dps_players.player_rows.is_empty() {
                            event_manager.emit_players_update(MetricType::Dps, dps_players);
                        }

                        // Generate and emit heal players update only if there's data
                        let heal_players = generate_players_window_heal(&encounter_state);
                        if !heal_players.player_rows.is_empty() {
                            event_manager.emit_players_update(MetricType::Heal, heal_players);
                        }

                        // Update skills store for all players with damage/heal data
                        let skills_store_state = app_handle.state::<SkillsStoreMutex>();
                        let mut skills_store = skills_store_state.lock().unwrap();

                        for (&entity_uid, entity) in &encounter_state.entity_uid_to_entity {
                            let is_player =
                                entity.entity_type == blueprotobuf::EEntityType::EntChar;
                            let has_dmg_skills = !entity.skill_uid_to_dmg_skill.is_empty();
                            let has_heal_skills = !entity.skill_uid_to_heal_skill.is_empty();

                            if is_player && has_dmg_skills {
                                if let Some(skills_window) =
                                    generate_skills_window_dps(&encounter_state, entity_uid)
                                {
                                    skills_store.update_dps_skills(entity_uid, skills_window);
                                }
                            }

                            if is_player && has_heal_skills {
                                if let Some(skills_window) =
                                    generate_skills_window_heal(&encounter_state, entity_uid)
                                {
                                    skills_store.update_heal_skills(entity_uid, skills_window);
                                }
                            }
                        }

                        // Emit skills updates only for subscribed players
                        for (&entity_uid, _) in &encounter_state.entity_uid_to_entity {
                            if skills_store.is_subscribed(entity_uid, "dps") {
                                if let Some(skills_window) = skills_store.get_dps_skills(entity_uid) {
                                    event_manager.emit_skills_update(MetricType::Dps, entity_uid, skills_window.clone());
                                }
                            }
                            if skills_store.is_subscribed(entity_uid, "heal") {
                                if let Some(skills_window) = skills_store.get_heal_skills(entity_uid) {
                                    event_manager.emit_skills_update(MetricType::Heal, entity_uid, skills_window.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
