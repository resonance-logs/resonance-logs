use log::info;
use crate::packets;

pub async fn start() { // todo: add app_handle?
    // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
    // 1. Start capturing packets
    let mut rx = packets::packet_capture::start_capture(); // Since live meter is not critical, it's ok to just log it // TODO: maybe bubble an error up to the frontend instead?

    // 2. Use the channel to receive packets back and process them
    while let Some((op, data)) = rx.recv().await {
        match op {
            packets::opcodes::Pkt::SyncNearEntities => {
                info!("Received NotifyMsg with opcode {op:?} and data {data:?}");
            }
            packets::opcodes::Pkt::DataNotifySyncContainerData => {
                info!("Received NotifyMsg with opcode {op:?} and data {data:?}");
            }
            packets::opcodes::Pkt::SyncContainerDirtyData => {
                info!("Received NotifyMsg with opcode {op:?} and data {data:?}");
            }
            packets::opcodes::Pkt::SyncServerTime => {
                info!("Received NotifyMsg with opcode {op:?} and data {data:?}");
            }
            packets::opcodes::Pkt::SyncToMeDeltaInfo => {
                info!("Received NotifyMsg with opcode {op:?} and data {data:?}");
                // let sync_to_me_delta_info =
                //     blueprotobuf::SyncToMeDeltaInfo::decode(bytes::Bytes::from(data))?;
                // let aoi_sync_to_me_delta = sync_to_me_delta_info.delta_info.unwrap_or_default();
                // let other_uuid = aoi_sync_to_me_delta.uuid.unwrap_or_default();
                // if my_uuid == 0 || my_uuid != other_uuid {
                //     info!("my uuid {:?}, uid: {:?}", other_uuid, other_uuid >> 16);
                //     my_uuid = other_uuid;
                // }
                //
                // if let Some(base_delta) = aoi_sync_to_me_delta.base_delta {
                //     process_aoi_sync_delta(base_delta);
                // }
            }
            packets::opcodes::Pkt::SyncNearDeltaInfo => {
                info!("Received NotifyMsg with opcode {op:?} and data {data:?}");
                // let sync_near_delta_info =
                //     blueprotobuf::SyncNearDeltaInfo::decode(bytes::Bytes::from(data))?;
                // let aoi_sync_delta_vec = sync_near_delta_info.delta_infos;
                // for aoi_sync_delta in aoi_sync_delta_vec {
                //     process_aoi_sync_delta(aoi_sync_delta);
                // }
            }
        }
    }
}
