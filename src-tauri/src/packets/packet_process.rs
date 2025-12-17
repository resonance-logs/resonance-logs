use crate::packets;
use crate::packets::opcodes::FragmentType;
use crate::packets::packet_event::PacketEvent;
use crate::packets::parser;
use crate::packets::utils::BinaryReader;
use log::{debug, info};

pub fn process_packet(
    mut packets_reader: BinaryReader,
    packet_sender: tokio::sync::mpsc::Sender<PacketEvent>,
) {
    let mut _debug_ctr = 0;
    while packets_reader.remaining() > 0 {
        let packet_size = match packets_reader.peek_u32() {
            Ok(sz) => sz,
            Err(e) => {
                debug!("Malformed packet: failed to peek_u32: {e}");
                continue;
            }
        };
        if packet_size < 6 {
            debug!("Malformed packet: packet_size < 6");
            continue;
        }

        let mut reader = match packets_reader.read_bytes(packet_size as usize) {
            Ok(bytes) => BinaryReader::from(bytes),
            Err(e) => {
                debug!("Malformed packet: failed to read_bytes: {e}");
                continue;
            }
        };
        if reader.read_u32().is_err() {
            debug!("Malformed packet: failed to skip u32");
            continue;
        }
        let packet_type = match reader.read_u16() {
            Ok(pt) => pt,
            Err(e) => {
                debug!("Malformed packet: failed to read_u16: {e}");
                continue;
            }
        };
        let is_zstd_compressed = packet_type & 0x8000;
        let msg_type_id = packet_type & 0x7fff;

        _debug_ctr += 1;
        match packets::opcodes::FragmentType::from(msg_type_id) {
            FragmentType::Notify => {
                // Use parser helper to extract components and payload.
                if let Some((method_id, payload)) =
                    parser::parse_notify_fragment(&mut reader, is_zstd_compressed != 0)
                {
                    if let Err(err) = packet_sender.try_send(PacketEvent::Notify {
                        op: method_id,
                        data: payload,
                    }) {
                        debug!("Failed to send packet (try_send): {err}");
                    }
                } else {
                    // parse_notify_fragment logged details
                    continue;
                }
            }
            FragmentType::FrameDown => {
                debug!(target: "app::market", "FrameDown packet received (compressed={})", is_zstd_compressed != 0);
                let server_sequence_id = match reader.read_u32() {
                    Ok(sid) => sid,
                    Err(_e) => {
                        debug!(target: "app::market", "FrameDown: failed to read server_sequence_id");
                        continue;
                    }
                };
                debug!(target: "app::market", "FrameDown server_sequence_id={}", server_sequence_id);
                if reader.remaining() == 0 {
                    debug!(target: "app::market", "FrameDown: no payload remaining");
                    break;
                }

                let nested_packet = reader.read_remaining();
                debug!(target: "app::market", "FrameDown nested_packet len={}", nested_packet.len());

                let nested_bytes = if is_zstd_compressed != 0 {
                    match zstd::decode_all(nested_packet) {
                        Ok(v) => {
                            debug!(target: "app::market", "FrameDown: zstd decompressed {} -> {} bytes", nested_packet.len(), v.len());
                            v
                        }
                        Err(e) => {
                            debug!(target: "app::market", "FrameDown: zstd decompression failed: {}", e);
                            continue;
                        }
                    }
                } else {
                    Vec::from(nested_packet)
                };

                // Quick visibility for debugging: market-like payloads typically contain
                // multiple 0x0A length-delimited fields early in the buffer.
                // (Keep this lightweight to avoid spamming.)
                if cfg!(debug_assertions) {
                    let preview_len = nested_bytes.len().min(2048);
                    let count_0a = nested_bytes[..preview_len]
                        .iter()
                        .filter(|&&b| b == 0x0A)
                        .count();
                    if count_0a >= 3 {
                        println!(
                            "[Market] FrameDown candidate: server_seq={} nested_len={} 0x0A(first{}B)={}",
                            server_sequence_id,
                            nested_bytes.len(),
                            preview_len,
                            count_0a
                        );
                    }
                }

                // FrameDown can contain either:
                // 1) a nested framed packet stream (combat/notify), or
                // 2) a protobuf-ish stream used by market replies.
                //
                // Try market decode first.
                debug!(target: "app::market", "FrameDown: attempting market decode on {} bytes", nested_bytes.len());
                if let Some(batch) = crate::packets::market_decode::decode_exchange_notice_detail(
                    &nested_bytes,
                    server_sequence_id,
                ) {
                    info!(target: "app::market", "FrameDown: market decode SUCCESS, {} listings", batch.listings.len());
                    if let Err(err) = packet_sender.try_send(PacketEvent::MarketListings(batch)) {
                        debug!("Failed to send market listings (try_send): {err}");
                    }
                    // Don't continue - we successfully decoded market data
                } else {
                    debug!(target: "app::market", "FrameDown: market decode returned None");
                    // Market decode failed, check if this might be nested framed packets
                    let looks_framed =
                        crate::packets::market_decode::looks_like_framed_packet_stream(
                            &nested_bytes,
                        );
                    debug!(target: "app::market", "FrameDown: looks_like_framed_packet_stream={}", looks_framed);
                    if looks_framed {
                        packets_reader = BinaryReader::from(nested_bytes);
                        continue;
                    }
                }
            }
            _ => {
                continue;
            }
        }
    }
}

// pub async fn process_packet(
//     mut tcp_fragments: BinaryReader,
//     packet_sender: tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>,
// ) {
//     println!("during process packet");
//     let mut debug_ctr = 0;
//     const MIN_FRAG_LEN: usize = 8 + 1 + 3; // frag_len + is_zstd + frag_type
//     println!("{}", tcp_fragments.remaining());
//     while tcp_fragments.remaining() >= MIN_FRAG_LEN {
//         let tcp_frag_len = tcp_fragments.peek_u32().unwrap();
//         if tcp_fragments.remaining() < tcp_frag_len as usize {
//             println!("{} < {tcp_frag_len}", tcp_fragments.remaining());
//             return;
//         }
//         let mut tcp_fragment = BinaryReader::from(tcp_fragments.read_bytes(tcp_frag_len as usize).unwrap());
//         let _ = tcp_fragment.read_u32(); // skip tcp_frag_len from before // todo: somehow this crashed before
//
//
//
//         let (is_zstd, frag_type) = {
//             let temp = tcp_fragment.read_u16().unwrap(); // todo: fix all these unwraps properly
//             ((temp & 0x8000) != 0, packets::opcodes::FragmentType::from(temp & 0x7fff)) // get bit 1 and bits 2-16
//         };
//
//         debug_ctr += 1;
//         println!("{frag_type:?}");
//         match frag_type {
//             packets::opcodes::FragmentType::Notify => {
//                 println!("{debug_ctr} Notify {:?}", tcp_fragment.cursor.get_ref());
//                 let service_uuid = tcp_fragment.read_u64().unwrap(); // service_uuid?
//                 let _stub_id = tcp_fragment.read_bytes(4); // bytes 15-18 are ignored
//
//                 if service_uuid == 63_335_342 {
//                     trace!("Skipping FragmentType with service_uuid: {service_uuid}");
//                     return;
//                 }
//
//                 let Ok(method_id) = packets::opcodes::Pkt::try_from(tcp_fragment.read_u32().unwrap()) else {
//                     return;
//                 };
//
//                 let mut tcp_fragment_vec = tcp_fragment.read_remaining().to_vec();
//                 if is_zstd {
//                     if let Ok(decoded) = zstd::decode_all(tcp_fragment_vec.as_slice()) {
//                         tcp_fragment_vec = decoded;
//                     } else {
//                         return; // faulty TCP packet
//                     }
//                 }
//
//                 if let Err(err) = packet_sender.send((method_id, tcp_fragment_vec)).await
//                 {
//                     debug!("Failed to send packet: {err}");
//                 }
//                 break;
//             }
//             packets::opcodes::FragmentType::FrameDown => {
//                 println!("{debug_ctr} FrameDown {:?}", tcp_fragment.cursor.get_ref());
//                 let _ = tcp_fragment.read_bytes(4).unwrap(); // bytes 1-4 are ignored
//                 let tcp_fragment_t = tcp_fragment.read_remaining(); // todo: change name
//                 if is_zstd {
//                     let Ok(tcp_fragment_decompressed) = zstd::decode_all(tcp_fragment_t) else {return};
//                     tcp_fragment.splice_remaining(&tcp_fragment_decompressed);
//                 }
//
//                 // recursively process the packet
//             }
//             _ => return,
//         }
//     }
// }

// todo: remove this test
#[cfg(test)]
mod tests {
    use crate::packets::packet_event::PacketEvent;
    use crate::packets::packet_process::process_packet;
    use crate::packets::utils::BinaryReader;

    #[tokio::test]
    async fn test_add() {
        use std::fs;
        let (packet_sender, _) = tokio::sync::mpsc::channel::<PacketEvent>(1);
        let filename = "src/packets/test_add_packet.json";
        let v: Vec<u8> = serde_json::from_str(
            &fs::read_to_string(filename).expect(&format!("Failed to open {filename}")),
        )
        .expect("Invalid JSON in test_packet.json");
        process_packet(BinaryReader::from(v), packet_sender);
    }
}
