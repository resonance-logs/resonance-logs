# Blueprotobuf Message Decoding

This document explains how the project ingests Blue Protocol network packets and decodes them into strongly typed protobuf messages using the `blueprotobuf` crate. It focuses on the Rust backend flow, the generated message types, and the downstream processing that extracts game semantics from the decoded structures.

## Generated crate layout

`blueprotobuf-lib` is a local workspace crate that exposes all generated protobuf definitions under the module `blueprotobuf` by including pre-built Prost and pbjson outputs. The crate simply re-exports the generated files and adds a helper conversion for entity typing, so the build script currently does not regenerate code at compile time.@src-tauri/src/blueprotobuf-lib/src/lib.rs#1-19

- `blueprotobuf_package.rs` – Prost-generated message structs implementing `prost::Message` plus `specta::Type` for TypeScript binding generation.
- `blueprotobuf_package.serde.rs` – pbjson-generated Serde implementations, enabling JSON serialization of the same messages.
- `impl From<i64> for EEntityType` – maps packed UUIDs to entity categories by masking the low 16 bits, which is used extensively during encounter processing.@src-tauri/src/blueprotobuf-lib/src/lib.rs#9-18

## Packet pipeline leading to protobuf decoding

1. **Capture & reassembly** – WinDivert captures raw TCP packets, which are reassembled and framed into protocol fragments. This step produces message payloads paired with Blue Protocol opcode identifiers for later routing.@src-tauri/src/packets/packet_capture.rs#18-225@src-tauri/src/packets/packet_process.rs#7-93
2. **Fragment parsing** – For `FragmentType::Notify` fragments, the parser ensures the expected service UUID, optionally decompresses zstd payloads, and yields `(Pkt, Vec<u8>)` pairs where the byte vector is the protobuf payload of a specific RPC.@src-tauri/src/packets/parser.rs#8-33
3. **Event dispatch** – `live_main::start` receives the opcode/data pairs, matches on the opcode, and uses `prost::Message::decode` on the raw bytes to build strongly typed protobuf structs. Successful decodes are wrapped in `StateEvent` variants for further handling; failures log warnings and the packet is skipped.@src-tauri/src/live/live_main.rs#26-101

This flow guarantees that any logic consuming `StateEvent` operates on validated protobuf data rather than raw bytes.

## Decoding entry points

The `blueprotobuf` module provides one struct per protobuf message. The decoding pattern is uniform:

```rust
match blueprotobuf::SyncNearEntities::decode(Bytes::from(data)) {
    Ok(message) => StateEvent::SyncNearEntities(message),
    Err(e) => { warn!(...); continue; }
}
```

Every handled opcode in `live_main` follows this pattern, resulting in six high-frequency protobuf messages being fed into the state machine: `SyncNearEntities`, `SyncContainerData`, `SyncContainerDirtyData`, `SyncServerTime`, `SyncToMeDeltaInfo`, and `SyncNearDeltaInfo`.@src-tauri/src/live/live_main.rs#31-85

Once decoded, `AppStateManager::handle_event` delegates to opcode-specific processing routines that interpret the message contents and mutate encounter state accordingly.@src-tauri/src/live/state.rs#129-201

## Working with message structures

### Entity appearance (`SyncNearEntities`)

`process_sync_near_entities` iterates through the decoded `appear` list, derives the stable entity ID by shifting the 64-bit UUID, and looks up or creates the corresponding encounter entity. It then dispatches to attribute processors based on entity type and persists player metadata to the database when available.@src-tauri/src/live/opcodes_process.rs#55-99

### Attribute decoding

Player and monster attributes are embedded as repeated `Attr` records inside protobuf collections. Because most attribute values are stored as opaque byte blobs, the project performs targeted manual decoding:

- Player attributes such as name, profession, level, combat stats, and resource pools are read by interpreting the raw bytes as protobuf-varint-encoded integers (or UTF-8 strings) and updating the in-memory `Entity` record accordingly.@src-tauri/src/live/opcodes_process.rs#571-920
- Monster attributes decode similarly, capturing monster IDs, localized names, and HP pools for later damage attribution.@src-tauri/src/live/opcodes_process.rs#1096-1132

Helper methods on `AttrValue` facilitate type-safe extraction of stored values for downstream calculations.@src-tauri/src/live/opcodes_models.rs#236-335

### Delta updates (`SyncNearDeltaInfo` / `SyncToMeDeltaInfo`)

Delta messages reuse the same attribute-processing helpers but also carry combat events (`SkillEffect` → `SyncDamageInfo`). `process_aoi_sync_delta` uses the decoded structures to distribute damage across attackers and defenders, apply boss-only filtering, and enqueue persistence tasks for damage logs.@src-tauri/src/live/opcodes_process.rs#200-540

## JSON and Specta integration

While the live pipeline consumes protobuf messages directly, the generated types also derive Serde and Specta traits. This enables:

- JSON serialization (via pbjson) when emitting data to other subsystems or storing debug dumps.
- TypeScript bindings for any Tauri commands or events that expose protobuf-backed data to the frontend.

These capabilities come “for free” from the generated modules and require no extra glue code in the crate.@src-tauri/src/blueprotobuf-lib/src/lib.rs#1-6

## Key takeaways

- `blueprotobuf` is a thin wrapper over generated Prost types; decoding is centralized in `live_main` and only performed after packet reassembly and validation.
- Manual attribute parsing is necessary because many protobuf fields are opaque byte blobs that need higher-level interpretation before they can populate encounter state.
- Entity typing derives from UUID bit patterns, enabling the system to route decoded data through player- or monster-specific logic without consulting additional metadata.
- The same message structs power both runtime processing and serialization/binding generation, ensuring a single source of truth for packet schemas across the project.
