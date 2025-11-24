use crate::live::event_manager::MetricType;
use crate::live::state::{AppStateManager, StateEvent};
use crate::packets;
use blueprotobuf_lib::blueprotobuf;
use bytes::Bytes;
use log::{info, trace, warn};
use prost::Message;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

/// Starts the live meter.
///
/// This function captures packets, processes them, and emits events to the frontend.
///
/// # Arguments
///
/// * `app_handle` - A handle to the Tauri application instance.
pub async fn start(app_handle: AppHandle) {
    // Get the state manager from app state
    let state_manager = app_handle.state::<AppStateManager>().inner().clone();

    // Initialize event manager - this should be done through the state manager now
    {
        // Initialize the event manager through the state manager
        let mut state = state_manager.state.write().await;
        state.event_manager.initialize(app_handle.clone());
    }

    // Throttling for events (emit at most every 200ms)
    let mut last_emit_time = Instant::now();
    let emit_throttle_duration = Duration::from_millis(200);

    // Heartbeat: ensure we emit events periodically even during idle periods
    // to prevent frontend from thinking the connection is dead
    let heartbeat_duration = Duration::from_secs(2);

    // 1. Start capturing packets and send to rx
    let mut rx = packets::packet_capture::start_capture(app_handle.clone()); // Since live meter is not critical, it's ok to just log it // TODO: maybe bubble an error up to the frontend instead?

    // 2. Use the channel to receive packets back and process them
    loop {
        // Use tokio::time::timeout to ensure we emit periodically even if no packets arrive
        let packet_result = tokio::time::timeout(heartbeat_duration, rx.recv()).await;

        // Helper to decode op/data into a StateEvent; returns None if decoding failed
        let decode_event = |op: packets::opcodes::Pkt, data: Vec<u8>| -> Option<StateEvent> {
            match op {
                packets::opcodes::Pkt::ServerChangeInfo => Some(StateEvent::ServerChange),
                packets::opcodes::Pkt::EnterScene => {
                    info!("Received EnterScene packet");
                    match blueprotobuf::EnterScene::decode(Bytes::from(data)) {
                        Ok(v) => Some(StateEvent::EnterScene(v)),
                        Err(e) => {
                            warn!("Error decoding EnterScene.. ignoring: {e}");
                            None
                        }
                    }
                }
                packets::opcodes::Pkt::SyncNearEntities => {
                    match blueprotobuf::SyncNearEntities::decode(Bytes::from(data)) {
                        Ok(v) => Some(StateEvent::SyncNearEntities(v)),
                        Err(e) => {
                            warn!("Error decoding SyncNearEntities.. ignoring: {e}");
                            None
                        }
                    }
                }
                packets::opcodes::Pkt::SyncContainerData => {
                    match blueprotobuf::SyncContainerData::decode(Bytes::from(data)) {
                        Ok(v) => Some(StateEvent::SyncContainerData(v)),
                        Err(e) => {
                            warn!("Error decoding SyncContainerData.. ignoring: {e}");
                            None
                        }
                    }
                }
                packets::opcodes::Pkt::SyncContainerDirtyData => {
                    match blueprotobuf::SyncContainerDirtyData::decode(Bytes::from(data)) {
                        Ok(v) => Some(StateEvent::SyncContainerDirtyData(v)),
                        Err(e) => {
                            warn!("Error decoding SyncContainerDirtyData.. ignoring: {e}");
                            None
                        }
                    }
                }
                packets::opcodes::Pkt::SyncServerTime => {
                    match blueprotobuf::SyncServerTime::decode(Bytes::from(data)) {
                        Ok(v) => Some(StateEvent::SyncServerTime(v)),
                        Err(e) => {
                            warn!("Error decoding SyncServerTime.. ignoring: {e}");
                            None
                        }
                    }
                }
                packets::opcodes::Pkt::SyncToMeDeltaInfo => {
                    match blueprotobuf::SyncToMeDeltaInfo::decode(Bytes::from(data)) {
                        Ok(v) => Some(StateEvent::SyncToMeDeltaInfo(v)),
                        Err(e) => {
                            warn!("Error decoding SyncToMeDeltaInfo.. ignoring: {e}");
                            None
                        }
                    }
                }
                packets::opcodes::Pkt::SyncNearDeltaInfo => {
                    match blueprotobuf::SyncNearDeltaInfo::decode(Bytes::from(data)) {
                        Ok(v) => Some(StateEvent::SyncNearDeltaInfo(v)),
                        Err(e) => {
                            warn!("Error decoding SyncNearDeltaInfo.. ignoring: {e}");
                            None
                        }
                    }
                }
                packets::opcodes::Pkt::NotifyReviveUser => {
                    match blueprotobuf::NotifyReviveUser::decode(Bytes::from(data)) {
                        Ok(v) => Some(StateEvent::NotifyReviveUser(v)),
                        Err(e) => {
                            warn!("Error decoding NotifyReviveUser.. ignoring: {e}");
                            None
                        }
                    }
                }
                packets::opcodes::Pkt::SyncSceneAttrs => {
                    match blueprotobuf::SyncSceneAttrs::decode(Bytes::from(data)) {
                        Ok(v) => Some(StateEvent::SyncSceneAttrs(v)),
                        Err(e) => {
                            warn!("Error decoding SyncSceneAttrs.. ignoring: {e}");
                            None
                        }
                    }
                }
                _ => {
                    trace!("Unhandled packet opcode: {op:?}");
                    None
                }
            }
        };

        match packet_result {
            Ok(Some((op, data))) => {
                // Process the first packet immediately (low-latency path)
                if let Some(event) = decode_event(op, data) {
                    state_manager.handle_event(event).await;
                }

                // Drain additional queued packets quickly but with a strict time budget
                // to avoid long stalls processing large backlogs. This keeps responsiveness
                // (e.g., ServerChange) while still improving throughput during bursts.
                let drain_start = Instant::now();
                let drain_time_budget = Duration::from_millis(10); // small budget to limit latency
                const MAX_DRAIN: usize = 256; // hard cap
                let mut drained = 0usize;

                loop {
                    if drained >= MAX_DRAIN {
                        break;
                    }
                    if Instant::now().duration_since(drain_start) >= drain_time_budget {
                        break;
                    }

                    match rx.try_recv() {
                        Ok((op, data)) => {
                            if let Some(event) = decode_event(op, data) {
                                // If a ServerChange is encountered, handle it and break quickly
                                let is_server_change = matches!(event, StateEvent::ServerChange);
                                state_manager.handle_event(event).await;
                                drained += 1;
                                if is_server_change {
                                    break;
                                }
                            } else {
                                drained += 1;
                            }
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            warn!("Packet capture channel closed (disconnected) while draining");
                            break;
                        }
                    }
                }

                // Check if we should emit events (throttling)
                let now = Instant::now();
                if now.duration_since(last_emit_time) >= emit_throttle_duration {
                    last_emit_time = now;
                    state_manager.update_and_emit_events().await;
                }
            }
            Ok(None) => {
                // Channel closed, exit loop
                warn!("Packet capture channel closed, exiting live meter loop");
                break;
            }
            Err(_) => {
                // Timeout occurred - no packets received within heartbeat_duration
                // Emit events anyway to keep frontend connection alive
                let now = Instant::now();
                if now.duration_since(last_emit_time) >= emit_throttle_duration {
                    last_emit_time = now;
                    state_manager.update_and_emit_events().await;
                }
            }
        }
    }
}
