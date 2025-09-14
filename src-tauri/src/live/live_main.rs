use crate::live::opcodes_process::{on_server_change, process_aoi_sync_delta, process_sync_to_me_delta_info};
use crate::packets;
use blueprotobuf_lib::blueprotobuf;
use bytes::Bytes;
use log::warn;
use prost::{Message};
use tauri::{AppHandle};

pub async fn start(app_handle: AppHandle) { // todo: add app_handle?
    // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
    // 1. Start capturing packets
    let mut rx = packets::packet_capture::start_capture(); // Since live meter is not critical, it's ok to just log it // TODO: maybe bubble an error up to the frontend instead?

    // 2. Use the channel to receive packets back and process them
    while let Some((op, data)) = rx.recv().await {
        match op {
            packets::opcodes::Pkt::ServerChangeInfo => {
                on_server_change(&app_handle);
            }
            packets::opcodes::Pkt::SyncNearEntities => {
                // info!("Received NotifyMsg with opcode {op:?}");
                // trace!("Received NotifyMsg with opcode {op:?} and data {data:?}");
                let _sync_near_entities = match blueprotobuf::SyncNearEntities::decode(Bytes::from(data)) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Error decoding .. ignoring: {e}");
                        continue;
                    }
                };
            }
            packets::opcodes::Pkt::SyncContainerData => {
                // info!("Received NotifyMsg with opcode {op:?}");
                // trace!("Received NotifyMsg with opcode {op:?} and data {data:?}");
                let _data_notify_sync_container_data = match blueprotobuf::SyncContainerData::decode(Bytes::from(data)) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Error decoding .. ignoring: {e}");
                        continue;
                    }
                };
            }
            packets::opcodes::Pkt::SyncContainerDirtyData => {
                // info!("Received NotifyMsg with opcode {op:?}");
                // trace!("Received NotifyMsg with opcode {op:?} and data {data:?}");
                let _sync_container_dirty_data = match blueprotobuf::SyncContainerDirtyData::decode(Bytes::from(data)) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Error decoding .. ignoring: {e}");
                        continue;
                    }
                };
            }
            packets::opcodes::Pkt::SyncServerTime => {
                // info!("Received NotifyMsg with opcode {op:?}");
                // trace!("Received NotifyMsg with opcode {op:?} and data {data:?}");
                let _sync_server_time = match blueprotobuf::SyncServerTime::decode(Bytes::from(data)) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Error decoding .. ignoring: {e}");
                        continue;
                    }
                };
                // todo: this is skipped, not sure what info it has
            }
            packets::opcodes::Pkt::SyncToMeDeltaInfo => {
                // info!("Received NotifyMsg with opcode {op:?}");
                // trace!("Received NotifyMsg with opcode {op:?} and data {data:?}");
                let sync_to_me_delta_info = match blueprotobuf::SyncToMeDeltaInfo::decode(Bytes::from(data)) {
                    Ok(sync_to_me_delta_info) => sync_to_me_delta_info,
                    Err(e) => {
                        warn!("Error decoding .. ignoring: {e}");
                        continue;
                    }
                };
                process_sync_to_me_delta_info(&app_handle, sync_to_me_delta_info); // todo: ignore the option, maybe we want to log a trace?
            }
            packets::opcodes::Pkt::SyncNearDeltaInfo => {
                // info!("Received NotifyMsg with opcode {op:?}");
                // trace!("Received NotifyMsg with opcode {op:?} and data {data:?}");
                let sync_near_delta_info = match blueprotobuf::SyncNearDeltaInfo::decode(Bytes::from(data)) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Error decoding .. ignoring: {e}");
                        continue;
                    }
                };
                for aoi_sync_delta in sync_near_delta_info.delta_infos {
                    process_aoi_sync_delta(&app_handle, aoi_sync_delta); // todo: ignore the option, maybe we want to log a trace?
                }
            }
        }
    }
}
