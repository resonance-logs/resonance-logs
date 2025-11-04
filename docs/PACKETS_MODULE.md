# Packets Module

The `packets` module is the foundation of the Resonance Logs application's real-time capabilities. It is responsible for capturing raw network traffic, identifying game-related packets, reassembling them into a coherent stream, and forwarding the processed data to the `live` module for analysis.

## Packet Capture: `packet_capture.rs`

The core of the packet capture functionality resides in `src-tauri/src/packets/packet_capture.rs`. This file uses the `WinDivert` library, a powerful tool for capturing and manipulating network packets on Windows.

### Key Responsibilities:

1.  **WinDivert Initialization**: The `read_packets` function initializes a `WinDivert` handle with a filter (`"!loopback && ip && tcp"`) to capture only relevant TCP/IP traffic, ignoring local loopback communication.
2.  **Game Server Identification**: A significant challenge in packet sniffing is identifying which of the many network connections belongs to the game. The module employs a heuristic approach to find the game server by inspecting the payload of initial TCP packets for specific signatures that are known to be part of the game's login and scene-change process.
3.  **Restart Mechanism**: The packet capture loop can be restarted on demand. The `request_restart` function sends a signal via a `tokio::sync::watch` channel, which causes the main `read_packets` loop to terminate and restart. This is crucial for handling situations like the player changing game servers without needing to restart the entire application.

## TCP Stream Reassembly

TCP is a stream-based protocol, but packet capture tools like `WinDivert` provide packets as individual, discrete units (datagrams). These packets can arrive out of order, be duplicated, or be lost. To correctly interpret the game's data stream, these raw TCP segments must be reassembled into the correct order. The `packets` module uses a two-stage reassembly process:

1.  **TCP Segment Reassembly (`TCPReassembler`)**: The first stage, handled by the `TCPReassembler` struct (defined in `utils.rs`), is responsible for handling the complexities of the TCP protocol. It uses TCP sequence numbers to buffer out-of-order packets and reorder them into a correct, sequential stream of bytes.
2.  **Application-Layer Frame Reassembly (`Reassembler`)**: Once a sequential stream of bytes is available, the `Reassembler` (from `reassembler.rs`) takes over. The game's protocol has its own framing mechanism on top of TCP, where each message is prefixed with its length. The `Reassembler` reads this length prefix to determine message boundaries, allowing it to extract complete, individual game packets from the TCP stream.

## Packet Processing: `packet_process.rs`

After a complete game packet has been reassembled, it is passed to the `process_packet` function in `src-tauri/src/packets/packet_process.rs`. This function:

1.  **Reads the Packet Header**: It parses the header of the game packet to identify the `opcode`, which is a unique number that signifies the type of the message (e.g., `EnterScene`, `SyncNearEntities`).
2.  **Forwards to the Live Module**: It creates a tuple containing the opcode and the raw packet data (`(Pkt, Vec<u8>)`).
3.  **Sends via Channel**: It sends this tuple through a `tokio::sync::mpsc::channel` to the `live` module's main processing loop (`live_main.rs`), which then decodes and acts upon the data.

This handoff decouples the packet capture and processing logic from the higher-level application state management and event handling logic, leading to a more modular and maintainable codebase.
