use std::io::{Cursor};
use bytes::Bytes;
use crate::packets::utils::{BinaryReader, Server};

use etherparse::NetSlice::Ipv4;
use etherparse::SlicedPacket;
use etherparse::TransportSlice::Tcp;
use log::{debug, info, trace, warn};
use windivert::prelude::WinDivertFlags;
use windivert::WinDivert;
use crate::packets;
use crate::packets::packet_process::process_packet;
use crate::packets::utils::TCPReassembler;

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
            let mut offset = 10; // Start at offset 10
            let tcp_payload = tcp_packet.payload();

            // 1. 5th byte from offset = Scene change?
            if let Some(0x00) = tcp_payload.get(offset + 4) { // todo: code can probably be cleaner
                const FRAG_LENGTH_SIZE: usize = 4;
                const SIGNATURE: [u8; 6] = [0x00, 0x63, 0x33, 0x53, 0x42, 0x00];

                let tcp_payload_len = tcp_payload.len();
                while offset + FRAG_LENGTH_SIZE <= tcp_payload_len { // todo: optimize: probs can use .chunks() https://users.rust-lang.org/t/best-option-to-read-from-slices-with-eof-cursor-is-bad-i-guess/68156
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

        // 2. TCP Packet Reconstruction todo: clean up? there's some stuff in the original about _data.length > 4 that i dont think is needed?
        // todo: tbh idk why in original meter this isnt done before finding the server address
        if let Some((seq_num, tcp_payload)) = tcp_reassembler.push_segment(tcp_packet.clone()) {
            trace!("Reassembled: Seq - {} - {}", seq_num, hex::encode(tcp_payload.clone())); // todo: comment for trace
            process_packet(BinaryReader::new(tcp_payload), packet_sender.clone()).await; // todo: optimize: instead of cloning, is it better to just move it to the function and return?
        }
    } // todo: if it errors, it breaks out of the loop but will it ever error?
}
