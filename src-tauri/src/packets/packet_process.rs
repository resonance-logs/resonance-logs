use std::io::Write;
use log::{debug, info};
use crate::packets;
use crate::packets::utils::BinaryReader;

pub async fn process_packet(
    mut tcp_fragments: BinaryReader,
    packet_sender: tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>,
) {
    const MIN_FRAG_SIZE: usize = 14; // frag_len + 4 ignored bytes + is_zstd + frag_type
    while tcp_fragments.remaining() >= MIN_FRAG_SIZE {
        let _ = tcp_fragments.read_bytes(4); // bytes 1-4 are ignored - frag_len

        let (is_zstd, frag_type) = {
            let temp = tcp_fragments.read_u16().unwrap();
            ((temp & 0x8000) != 0, packets::opcodes::FragmentType::from(temp & 0x7fff)) // get bit 1 and bits 2-16
        };

        match frag_type {
            packets::opcodes::FragmentType::FrameDown => {
                let _ = tcp_fragments.read_bytes(4).unwrap(); // bytes 1-4 are ignored
                let tcp_fragment = tcp_fragments.read_remaining();
                if is_zstd {
                    let Ok(tcp_fragment_decompressed) = zstd::decode_all(tcp_fragment) else {return};
                    tcp_fragments.splice_remaining(&tcp_fragment_decompressed);
                }
                // recursively process the packet
            }
            packets::opcodes::FragmentType::Notify => {
                let _ = tcp_fragments.read_bytes(8); // service_uuid?
                let _ = tcp_fragments.read_bytes(4);

                let Ok(method_id) = packets::opcodes::Pkt::try_from(tcp_fragments.read_u32().unwrap()) else {
                    return;
                };

                let tcp_fragment = tcp_fragments.read_remaining();
                let tcp_fragment_decompressed = if is_zstd {
                    match zstd::decode_all(tcp_fragment) {
                        Ok(decoded) => decoded,
                        Err(_) => return, // faulty TCP packet, just return
                    }
                } else {
                    tcp_fragment.to_vec()
                };

                if let Err(err) = packet_sender.send((method_id, tcp_fragment_decompressed)).await
                {
                    debug!("Failed to send packet: {err}");
                }
            }
            _ => return,
        }
    }
}
