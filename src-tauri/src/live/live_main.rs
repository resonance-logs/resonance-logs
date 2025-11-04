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

    // 1. Start capturing packets and send to rx
    let mut rx = packets::packet_capture::start_capture(); // Since live meter is not critical, it's ok to just log it // TODO: maybe bubble an error up to the frontend instead?

    // 2. Use the channel to receive packets back and process them
    while let Some((op, data)) = rx.recv().await {
        let event = match op {
            packets::opcodes::Pkt::ServerChangeInfo => StateEvent::ServerChange,
            packets::opcodes::Pkt::EnterScene => {
                info!("Received EnterScene packet");
                match blueprotobuf::EnterScene::decode(Bytes::from(data)) {
                    Ok(v) => StateEvent::EnterScene(v),
                    Err(e) => {
                        warn!("Error decoding EnterScene.. ignoring: {e}");
                        continue;
                    }
                }
            }
            packets::opcodes::Pkt::SyncNearEntities => {
                match blueprotobuf::SyncNearEntities::decode(Bytes::from(data)) {
                    Ok(v) => StateEvent::SyncNearEntities(v),
                    Err(e) => {
                        warn!("Error decoding SyncNearEntities.. ignoring: {e}");
                        continue;
                    }
                }
            }
            packets::opcodes::Pkt::SyncContainerData => {
                match blueprotobuf::SyncContainerData::decode(Bytes::from(data)) {
                    Ok(v) => StateEvent::SyncContainerData(v),
                    Err(e) => {
                        warn!("Error decoding SyncContainerData.. ignoring: {e}");
                        continue;
                    }
                }
            }
            packets::opcodes::Pkt::SyncContainerDirtyData => {
                match blueprotobuf::SyncContainerDirtyData::decode(Bytes::from(data)) {
                    Ok(v) => StateEvent::SyncContainerDirtyData(v),
                    Err(e) => {
                        warn!("Error decoding SyncContainerDirtyData.. ignoring: {e}");
                        continue;
                    }
                }
            }
            packets::opcodes::Pkt::SyncServerTime => {
                match blueprotobuf::SyncServerTime::decode(Bytes::from(data)) {
                    Ok(v) => StateEvent::SyncServerTime(v),
                    Err(e) => {
                        warn!("Error decoding SyncServerTime.. ignoring: {e}");
                        continue;
                    }
                }
            }
            packets::opcodes::Pkt::SyncToMeDeltaInfo => {
                match blueprotobuf::SyncToMeDeltaInfo::decode(Bytes::from(data)) {
                    Ok(v) => StateEvent::SyncToMeDeltaInfo(v),
                    Err(e) => {
                        warn!("Error decoding SyncToMeDeltaInfo.. ignoring: {e}");
                        continue;
                    }
                }
            }
            packets::opcodes::Pkt::SyncNearDeltaInfo => {
                match blueprotobuf::SyncNearDeltaInfo::decode(Bytes::from(data)) {
                    Ok(v) => StateEvent::SyncNearDeltaInfo(v),
                    Err(e) => {
                        warn!("Error decoding SyncNearDeltaInfo.. ignoring: {e}");
                        continue;
                    }
                }
            }
            _ => {
                trace!("Unhandled packet opcode: {op:?}");
                continue;
            }
        };

        // Handle the event
        state_manager.handle_event(event).await;

        // Check if we should emit events (throttling)
        let now = Instant::now();
        if now.duration_since(last_emit_time) >= emit_throttle_duration {
            last_emit_time = now;
            state_manager.update_and_emit_events().await;
        }
    }
}
