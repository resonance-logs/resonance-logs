use etherparse::NetSlice::Ipv4;
use etherparse::SlicedPacket;
use etherparse::TransportSlice::Tcp;
use log::{info, trace, warn};
use windivert::prelude::WinDivertFlags;
use windivert::WinDivert;
use crate::packets;
use crate::packets::packet_process::process_packet;
use crate::packets::utils::{TCPReassembler, BinaryReader, Server};

pub fn start_capture() -> tokio::sync::mpsc::Receiver<(packets::opcodes::Pkt, Vec<u8>)> {
    let (packet_sender, packet_receiver) = tokio::sync::mpsc::channel::<(packets::opcodes::Pkt, Vec<u8>)>(1);
    tauri::async_runtime::spawn(async move { read_packets(packet_sender).await });
    packet_receiver
}

async fn read_packets(packet_sender: tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>) {
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
        let Ok(network_slices) = SlicedPacket::from_ip(packet.data.as_ref()) else {
            continue; // if it's not ip, go next packet
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
            let tcp_payload = tcp_packet.payload();
            // 1. 5th byte from offset = Scene change?
            let mut tcp_payload_reader = BinaryReader::from(tcp_payload.to_vec());
            if tcp_payload_reader.remaining() >= 5 && tcp_payload_reader.read_bytes(5).unwrap()[4] == 0 { // 5th byte has to be 0x00
                const FRAG_LENGTH_SIZE: usize = 4;
                const SIGNATURE: [u8; 6] = [0x00, 0x63, 0x33, 0x53, 0x42, 0x00];
                let _ = tcp_payload_reader.read_bytes(5); // Start at offset 10 (5+5)
                // let tcp_payload_len = tcp_payload_reader.len();
                while tcp_payload_reader.remaining() >= FRAG_LENGTH_SIZE {
                    // Read fragment length
                    let tcp_frag_payload_len = tcp_payload_reader.read_u32().unwrap().saturating_sub(FRAG_LENGTH_SIZE as u32) as usize;

                    if tcp_payload_reader.remaining() >= tcp_frag_payload_len {
                        let tcp_frag = tcp_payload_reader.read_bytes(tcp_frag_payload_len).unwrap();

                        if tcp_frag.len() >= 5 + SIGNATURE.len() && tcp_frag[5..5 + SIGNATURE.len()] == SIGNATURE {
                            info!("Got Scene Server Address (by change): {curr_server}");
                            known_server = Some(curr_server);
                            tcp_reassembler.clear_reassembler(tcp_packet.sequence_number() as usize + tcp_payload_reader.len());
                            // todo: clearDataOnServerChange();
                        }
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
                    info!("Got Scene Server Address by Login Return Packet: {curr_server}");
                    known_server = Some(curr_server);
                    tcp_reassembler.clear_reassembler(tcp_packet.sequence_number() as usize + tcp_payload.len());
                    // todo: clearDataOnServerChange();
                }
            }
        }

        // 2. TCP Packet Reconstruction todo: clean up? there's some stuff in the original about _data.length > 4 that i dont think is needed?
        // todo: tbh idk why in original meter this isnt done before finding the server address
        if let Some((seq_num, tcp_payload)) = tcp_reassembler.push_segment(tcp_packet.clone()) {
            trace!("Reassembled: Seq - {} - {}", seq_num, hex::encode(tcp_payload.clone())); // todo: comment for trace
            process_packet(BinaryReader::from(tcp_payload), packet_sender.clone()).await; // todo: optimize: instead of cloning, is it better to just move it to the function and return?
        }
    } // todo: if it errors, it breaks out of the loop but will it ever error?
}
