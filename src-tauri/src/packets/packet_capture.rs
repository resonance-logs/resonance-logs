use crate::live::state::{AppStateManager, CaptureMethod};
use crate::packets;
use crate::packets::opcodes::Pkt;
use crate::packets::packet_process::process_packet;
use crate::packets::reassembler::Reassembler;
use crate::packets::utils::{BinaryReader, Server, TCPReassembler, tcp_sequence_before};
use etherparse::NetSlice::Ipv4;
use etherparse::SlicedPacket;
use etherparse::TransportSlice::Tcp;
use log::{debug, error, info, warn};
use once_cell::sync::OnceCell;
use tauri::Manager;
use tokio::sync::watch;
use windivert::WinDivert;
use windivert::prelude::WinDivertFlags;

#[cfg(feature = "npcap")]
use pcap::Linktype;

// Global sender for restart signal
static RESTART_SENDER: OnceCell<watch::Sender<bool>> = OnceCell::new();

const MAX_BACKTRACK_BYTES: u32 = 2 * 1024 * 1024; // 2 MiB safety window before considering a reset

pub fn start_capture(
    app_handle: tauri::AppHandle,
) -> tokio::sync::mpsc::Receiver<(packets::opcodes::Pkt, Vec<u8>)> {
    // Use a larger bounded channel to prevent producer backpressure from stalling
    // headroom for bursts without risking unbounded memory growth.
    let (packet_sender, packet_receiver) =
        tokio::sync::mpsc::channel::<(packets::opcodes::Pkt, Vec<u8>)>(10);
    let (restart_sender, mut restart_receiver) = watch::channel(false);
    RESTART_SENDER.set(restart_sender.clone()).ok();

    tauri::async_runtime::spawn(async move {
        loop {
            read_packets(app_handle.clone(), &packet_sender, &mut restart_receiver).await;
            // Wait for restart signal
            while !*restart_receiver.borrow() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            // Reset signal to false before next loop
            let _ = restart_sender.send(false);
        }
        // info!("oopsies {}", line!());
    });
    packet_receiver
}

#[allow(clippy::too_many_lines)]
async fn read_packets(
    app_handle: tauri::AppHandle,
    packet_sender: &tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>,
    restart_receiver: &mut watch::Receiver<bool>,
) {
    let (method, adapter) = {
        let state_manager = app_handle.state::<AppStateManager>();
        let state = state_manager.state.read().await;
        (
            state.packet_capture_config.method.clone(),
            state.packet_capture_config.adapter.clone(),
        )
    };

    let mut known_server: Option<Server> = None; // nothing at start
    let mut tcp_reassembler: TCPReassembler = TCPReassembler::new();
    let mut reassembler = Reassembler::new();

    match method {
        CaptureMethod::Windivert => {
            let windivert = match WinDivert::network(
                "!loopback && ip && tcp", // todo: idk why but filtering by port just crashes the program, investigate?
                0,
                WinDivertFlags::new().set_sniff(),
            ) {
                Ok(windivert_handle) => {
                    info!("WinDivert handle opened!");
                    Some(windivert_handle)
                }
                Err(e) => {
                    error!("Failed to initialize WinDivert: {}", e);
                    return;
                }
            }
            .expect("Failed to initialize WinDivert"); // if windivert doesn't work just exit early - todo: maybe we want to log this with a match so its clearer?

            let mut windivert_buffer = vec![0u8; 10 * 1024 * 1024];

            while let Ok(packet) = windivert.recv(Some(&mut windivert_buffer)) {
                if let Ok(slices) = SlicedPacket::from_ip(packet.data.as_ref()) {
                    process_raw_packet(
                        slices,
                        packet_sender,
                        &mut known_server,
                        &mut tcp_reassembler,
                        &mut reassembler,
                    )
                    .await;
                }

                if *restart_receiver.borrow() {
                    break;
                }
            }
        }
        #[cfg(feature = "npcap")]
        CaptureMethod::Npcap => {
            let device_name = adapter.unwrap_or_else(|| "\\Device\\NPF_Loopback".to_string());
            info!("Initializing Npcap on device: {}", device_name);

            let mut cap = match pcap::Capture::from_device(device_name.as_str()) {
                Ok(d) => match d.immediate_mode(true).open() {
                    Ok(c) => c,
                    Err(e) => {
                        let error_msg = e.to_string();
                        if error_msg.contains("wpcap")
                            || error_msg.contains("packet.dll")
                            || error_msg.contains("library")
                        {
                            error!(
                                "Npcap is not installed or wpcap.dll could not be loaded. \
                                Please install Npcap from https://npcap.com/ to use network adapter selection. \
                                Error: {}",
                                e
                            );
                        } else {
                            error!("Failed to open Npcap device: {}", e);
                        }
                        return;
                    }
                },
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("wpcap")
                        || error_msg.contains("packet.dll")
                        || error_msg.contains("library")
                    {
                        error!(
                            "Npcap is not installed or wpcap.dll could not be loaded. \
                            Please install Npcap from https://npcap.com/ to use network adapter selection. \
                            Error: {}",
                            e
                        );
                    } else {
                        error!("Failed to find Npcap device: {}", e);
                    }
                    return;
                }
            };

            if let Err(e) = cap.filter("tcp", true) {
                warn!("Failed to set pcap filter: {}", e);
            }

            let link_type = cap.get_datalink();
            info!("Npcap link type: {:?}", link_type);

            loop {
                match cap.next_packet() {
                    Ok(packet) => {
                        let sliced = if link_type == Linktype::ETHERNET {
                            SlicedPacket::from_ethernet(packet.data)
                        } else if link_type == Linktype(0) {
                            if packet.data.len() >= 4 {
                                SlicedPacket::from_ip(&packet.data[4..])
                            } else {
                                SlicedPacket::from_ip(&[])
                            }
                        } else {
                            SlicedPacket::from_ip(packet.data)
                        };

                        if let Ok(slices) = sliced {
                            process_raw_packet(
                                slices,
                                packet_sender,
                                &mut known_server,
                                &mut tcp_reassembler,
                                &mut reassembler,
                            )
                            .await;
                        }
                    }
                    Err(pcap::Error::TimeoutExpired) => {
                        // Continue loop on timeout
                    }
                    Err(e) => {
                        warn!("Npcap error: {}", e);
                        // break? or continue?
                    }
                }

                if *restart_receiver.borrow() {
                    break;
                }

                // Yield to runtime to allow other tasks (like restart signal check) to run
                // since pcap might be blocking or tight loop
                tokio::task::yield_now().await;
            }
        }
        #[cfg(not(feature = "npcap"))]
        CaptureMethod::Npcap => {
            error!("Npcap support not compiled in. Falling back to WinDivert.");
            warn!("To use Npcap, rebuild with --features npcap");
            // Fall back to WinDivert
            let windivert = match WinDivert::network(
                "!loopback && ip && tcp",
                0,
                WinDivertFlags::new().set_sniff(),
            ) {
                Ok(windivert_handle) => {
                    info!("WinDivert handle opened (fallback from Npcap)!");
                    Some(windivert_handle)
                }
                Err(e) => {
                    error!("Failed to initialize WinDivert fallback: {}", e);
                    return;
                }
            }
            .expect("Failed to initialize WinDivert");

            let mut windivert_buffer = vec![0u8; 10 * 1024 * 1024];

            while let Ok(packet) = windivert.recv(Some(&mut windivert_buffer)) {
                if let Ok(slices) = SlicedPacket::from_ip(packet.data.as_ref()) {
                    process_raw_packet(
                        slices,
                        packet_sender,
                        &mut known_server,
                        &mut tcp_reassembler,
                        &mut reassembler,
                    )
                    .await;
                }

                if *restart_receiver.borrow() {
                    break;
                }
            }
        }
    }
}

async fn process_raw_packet(
    network_slices: SlicedPacket<'_>,
    packet_sender: &tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>,
    known_server: &mut Option<Server>,
    tcp_reassembler: &mut TCPReassembler,
    reassembler: &mut Reassembler,
) {
    // let Ok(network_slices) = SlicedPacket::from_ip(packet_data) else {
    //     return; // if it's not ip, go next packet
    // };
    let Some(Ipv4(ip_packet)) = network_slices.net else {
        return;
    };
    let Some(Tcp(tcp_packet)) = network_slices.transport else {
        return;
    };

    let curr_server = Server::new(
        ip_packet.header().source(),
        tcp_packet.to_header().source_port,
        ip_packet.header().destination(),
        tcp_packet.to_header().destination_port,
    );

    // 1. Try to identify game server via small packets
    if *known_server != Some(curr_server) {
        let tcp_payload = tcp_packet.payload();
        let mut tcp_payload_reader = BinaryReader::from(tcp_payload.to_vec());
        if tcp_payload_reader.remaining() >= 10 {
            match tcp_payload_reader.read_bytes(10) {
                Ok(bytes) => {
                    if bytes[4] == 0 {
                        const FRAG_LENGTH_SIZE: usize = 4;
                        const SIGNATURE: [u8; 6] = [0x00, 0x63, 0x33, 0x53, 0x42, 0x00];
                        const MAX_FRAG_ITERATIONS: usize = 2000; // Circuit breaker

                        let mut i = 0;
                        while tcp_payload_reader.remaining() >= FRAG_LENGTH_SIZE {
                            i += 1;
                            if i >= MAX_FRAG_ITERATIONS {
                                error!(
                                    "TCP fragment processing stuck after {i} iterations - forcing recovery. \
                                    remaining={}, line={}",
                                    tcp_payload_reader.remaining(),
                                    line!()
                                );
                                break;
                            }
                            if i % 1000 == 0 {
                                warn!(
                                    "High iteration count in fragment processing: iteration={i}, remaining={}, line={}",
                                    tcp_payload_reader.remaining(),
                                    line!()
                                );
                            }
                            let tcp_frag_payload_len = match tcp_payload_reader.read_u32() {
                                Ok(len) => len.saturating_sub(FRAG_LENGTH_SIZE as u32) as usize,
                                Err(e) => {
                                    debug!("Malformed TCP fragment: failed to read_u32: {e}");
                                    break;
                                }
                            };
                            if tcp_payload_reader.remaining() >= tcp_frag_payload_len {
                                match tcp_payload_reader.read_bytes(tcp_frag_payload_len) {
                                    Ok(tcp_frag) => {
                                        if tcp_frag.len() >= 5 + SIGNATURE.len()
                                            && tcp_frag[5..5 + SIGNATURE.len()] == SIGNATURE
                                        {
                                            info!(
                                                "Got Scene Server Address (by change): {curr_server}"
                                            );
                                            *known_server = Some(curr_server);
                                            let payload_len =
                                                u32::try_from(tcp_payload_reader.len())
                                                    .unwrap_or(u32::MAX);
                                            let seq_end = tcp_packet
                                                .sequence_number()
                                                .wrapping_add(payload_len);
                                            reset_stream(
                                                tcp_reassembler,
                                                reassembler,
                                                Some(seq_end),
                                            );
                                            if let Err(err) = packet_sender
                                                .send((Pkt::ServerChangeInfo, Vec::new()))
                                                .await
                                            {
                                                debug!("Failed to send packet: {err}");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        debug!("Malformed TCP fragment: failed to read_bytes: {e}");
                                        break;
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("Malformed TCP payload: failed to read_bytes(10): {e}");
                }
            }
        }
        // 2. Payload length is 98 = Login packets?
        if tcp_payload.len() == 98 {
            const SIGNATURE_1: [u8; 10] =
                [0x00, 0x00, 0x00, 0x62, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01];
            const SIGNATURE_2: [u8; 6] = [0x00, 0x00, 0x00, 0x00, 0x0a, 0x4e];
            if tcp_payload.len() >= 20
                && tcp_payload[0..10] == SIGNATURE_1
                && tcp_payload[14..20] == SIGNATURE_2
            {
                info!("Got Scene Server Address by Login Return Packet: {curr_server}");
                *known_server = Some(curr_server);
                let payload_len = u32::try_from(tcp_payload.len()).unwrap_or(u32::MAX);
                let seq_end = tcp_packet.sequence_number().wrapping_add(payload_len);
                reset_stream(tcp_reassembler, reassembler, Some(seq_end));
                if let Err(err) = packet_sender
                    .send((Pkt::ServerChangeInfo, Vec::new()))
                    .await
                {
                    debug!("Failed to send packet: {err}");
                }
            }
        }
        return;
    }

    let sequence_number = tcp_packet.sequence_number();
    let payload = tcp_packet.payload();
    let payload_len = payload.len();

    if tcp_packet.syn() {
        info!("SYN observed for {curr_server}; resetting TCP reassembler state");
        reset_stream(
            tcp_reassembler,
            reassembler,
            Some(sequence_number.wrapping_add(1)),
        );
        if payload_len == 0 {
            return;
        }
    }

    let mut defer_reset = false;
    if tcp_packet.fin() || tcp_packet.rst() {
        defer_reset = true;
    }

    if payload_len == 0 {
        if defer_reset {
            reset_stream(tcp_reassembler, reassembler, None);
        }
        return;
    }

    if let Some(expected) = tcp_reassembler.next_sequence() {
        if tcp_sequence_before(sequence_number, expected) {
            let backwards = expected.wrapping_sub(sequence_number);
            if backwards > MAX_BACKTRACK_BYTES {
                warn!(
                    "Sequence regression detected for {curr_server}: expected {expected}, \
                    got {sequence_number} (backwards {backwards} bytes). Resetting stream"
                );
                reset_stream(tcp_reassembler, reassembler, Some(sequence_number));
            }
        }
    }

    if let Some(buffer) = tcp_reassembler.insert_segment(sequence_number, payload) {
        reassembler.feed_owned(buffer);
    }

    while let Some(packet) = reassembler.try_next() {
        process_packet(BinaryReader::from(packet), packet_sender.clone()).await;
    }

    if defer_reset {
        reset_stream(tcp_reassembler, reassembler, None);
    }
}

// Function to send restart signal from another thread/task
pub fn request_restart() {
    if let Some(sender) = RESTART_SENDER.get() {
        let _ = sender.send(true);
    }
}

fn reset_stream(
    tcp_reassembler: &mut TCPReassembler,
    reassembler: &mut Reassembler,
    next_seq: Option<u32>,
) {
    reassembler.take_remaining();
    tcp_reassembler.reset(next_seq);
}
