use crate::packets::opcodes::Pkt;

use crate::packets::utils::Server;

use etherparse::NetSlice::Ipv4;
use etherparse::SlicedPacket;
use etherparse::TransportSlice::Tcp;
use log::{debug, info, trace, warn};
use tokio::sync::mpsc::{channel, Receiver};
use windivert::prelude::WinDivertFlags;
use windivert::WinDivert;
use crate::packets::tcp_reassembler::TCPReassembler;

pub fn start_capture() -> Receiver<(Pkt, Vec<u8>)> {
    let (_packet_sender, packet_receiver) = channel::<(Pkt, Vec<u8>)>(1);
    tauri::async_runtime::spawn(async move { read_packets() });
    packet_receiver
}

fn read_packets() {
    let windivert = WinDivert::network(
        "!loopback && ip && tcp", // todo: idk why but filtering by port just crashes the program, investigate?
        0,
        WinDivertFlags::new().set_sniff(),
    ) // todo: add logs on windivert success
    .expect("Failed to initialize WinDivert"); // if windivert doesn't work just exit early - todo: maybe we want to log this with a match so its clearer?
    let mut windivert_buffer = vec![0u8; 10 * 1024 * 1024];
    let mut known_server: Option<Server> = None; // nothing at start
    let mut tcp_reassembler: TCPReassembler = TCPReassembler::new();
    while let Ok(packet) = windivert.recv(Some(&mut windivert_buffer)) {
        // todo: maybe from_ip instead?
        let Ok(network_slices) = SlicedPacket::from_ip(packet.data.as_ref()) else {
            continue; // if it's not ip, go next
        };
        let Some(Ipv4(ip_packet)) = network_slices.net else {
            continue;
        };
        let Some(Tcp(tcp_packet)) = network_slices.transport else {
            continue;
        };
        let curr_server = Server::new(
            ip_packet.header().source(),
            tcp_packet.to_header().source_port,
            ip_packet.header().destination(),
            tcp_packet.to_header().destination_port,
        );
        trace!(
            "{} ({}) => {:?}",
            curr_server,
            tcp_packet.payload().len(),
            hex::encode(tcp_packet.payload()),
        );

        // 1. Try to identify game server via small packets
        if known_server != Some(curr_server) {
            // let mut tcp_payload_bytes = tcp_packet.payload().to_vec();
            let mut offset = 10; // Start at offset 10 instead of slicing
            let tcp_payload = tcp_packet.payload();

            // 1. 5th byte from offset = Scene change?
            if let Some(0x00) = tcp_payload.get(offset + 4) { // todo: code can probably be cleaner
                const FRAG_LENGTH_SIZE: usize = 4;
                const SIGNATURE: [u8; 6] = [0x00, 0x63, 0x33, 0x53, 0x42, 0x00];

                let tcp_payload_len = tcp_payload.len();
                while offset + FRAG_LENGTH_SIZE <= tcp_payload_len {
                    // Read fragment length
                    let tcp_frag_payload_len = u32::from_be_bytes(
                        tcp_payload[offset..offset + FRAG_LENGTH_SIZE].try_into().unwrap()
                    ).saturating_sub(FRAG_LENGTH_SIZE as u32) as usize;
                    offset += FRAG_LENGTH_SIZE;

                    if offset + tcp_frag_payload_len > tcp_payload_len {
                        trace!(" continue 1"); // todo: comment
                        continue;
                    }

                    let tcp_frag = &tcp_payload[offset..offset + tcp_frag_payload_len];
                    offset += tcp_frag_payload_len;

                    if tcp_frag.len() >= 5 + SIGNATURE.len() && tcp_frag[5..5 + SIGNATURE.len()] == SIGNATURE {
                        info!("Got Scene Server Address (by change): {curr_server}");
                        known_server = Some(curr_server);
                        tcp_reassembler.clear_reassembler(tcp_packet.sequence_number() + tcp_payload.len() as u32);
                        // todo: clearDataOnServerChange();
                    }
                }
            }

            // 2. Payload length is 98 = Login packets?
            if tcp_payload.len() == 98 {
                const SIGNATURE_1: [u8; 10] = [
                    0x00, 0x00, 0x00, 0x62, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01,
                ];

                const SIGNATURE_2: [u8; 6] = [
                    0x00, 0x00, 0x00, 0x00, 0x0a, 0x4e,
                ];

                if tcp_payload.len() >= 20
                    && tcp_payload[0..10] == SIGNATURE_1
                    && tcp_payload[14..20] == SIGNATURE_2
                {
                    // reassembler.clear_cache(tcp_packet.sequence_number() + tcp_packet.payload().len() as u32);
                    known_server = Option::from(curr_server);
                    info!("Got Scene Server Address by Login Return Packet: {curr_server}");
                    tcp_reassembler.clear_reassembler(tcp_packet.sequence_number() + tcp_payload.len() as u32);
                    // todo: clearDataOnServerChange();
                }
            }
        }

        // 3. TCP Packet Reconstruction todo: clean up?
        if let Some((seq_num, data)) = tcp_reassembler.push_segment(tcp_packet.clone()) {
            trace!("Reassembled: Seq - {} - {}", seq_num, hex::encode(data)); // todo: comment for trace
            // let mut processor = PacketProcessor::new();
            // processor.process_packet_init(data, packet_sender.clone(), src_server);
        }
    } // todo: if it errors, it breaks out of the loop but will it ever error?
}
