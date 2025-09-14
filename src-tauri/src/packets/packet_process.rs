use log::{debug, trace};
use crate::packets;
use crate::packets::utils::BinaryReader;

pub async fn process_packet(
    mut tcp_fragments: BinaryReader,
    packet_sender: tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>,
) {
    const MIN_FRAG_LEN: usize = 8 + 1 + 3; // frag_len + is_zstd + frag_type
    while tcp_fragments.remaining() >= MIN_FRAG_LEN {
        let tcp_frag_len = tcp_fragments.peek_u32().unwrap();
        if tcp_fragments.remaining() < tcp_frag_len as usize {
            return;
        }
        tcp_fragments = BinaryReader::from(tcp_fragments.read_bytes(tcp_frag_len as usize).unwrap());
        let _tcp_frag_len = tcp_fragments.read_u32(); // skip tcp_frag_len from before

        let (is_zstd, frag_type) = {
            let temp = tcp_fragments.read_u16().unwrap(); // todo: fix all these unwraps properly
            ((temp & 0x8000) != 0, packets::opcodes::FragmentType::from(temp & 0x7fff)) // get bit 1 and bits 2-16
        };


        match frag_type {
            packets::opcodes::FragmentType::Notify => {
                let service_uuid = tcp_fragments.read_u64().unwrap(); // service_uuid?
                let _stub_id = tcp_fragments.read_bytes(4); // bytes 15-18 are ignored

                if service_uuid == 63_335_342 {
                    trace!("Skipping FragmentType with service_uuid: {service_uuid}");
                    return;
                }

                let Ok(method_id) = packets::opcodes::Pkt::try_from(tcp_fragments.read_u32().unwrap()) else {
                    return;
                };

                let mut tcp_fragment_vec = tcp_fragments.read_remaining().to_vec();
                if is_zstd {
                    if let Ok(decoded) = zstd::decode_all(tcp_fragment_vec.as_slice()) {
                        tcp_fragment_vec = decoded;
                    } else {
                        return; // faulty TCP packet
                    }
                }

                if let Err(err) = packet_sender.send((method_id, tcp_fragment_vec)).await
                {
                    debug!("Failed to send packet: {err}");
                }
            }
            packets::opcodes::FragmentType::FrameDown => {
                let _ = tcp_fragments.read_bytes(4).unwrap(); // bytes 1-4 are ignored
                let tcp_fragment_t = tcp_fragments.read_remaining(); // todo: change name
                if is_zstd {
                    let Ok(tcp_fragment_decompressed) = zstd::decode_all(tcp_fragment_t) else {return};
                    tcp_fragments.splice_remaining(&tcp_fragment_decompressed);
                }
                // recursively process the packet
            }
            _ => return,
        }
    }
}
