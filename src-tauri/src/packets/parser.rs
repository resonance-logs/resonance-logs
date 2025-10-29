use crate::packets;
use crate::packets::opcodes::Pkt;
use crate::packets::utils::BinaryReader;
use log::debug;

/// Parse a single notify fragment from `reader` (reader is advanced) and return the
/// method id and payload bytes. If compression is set, it is applied here.
pub fn parse_notify_fragment(
    reader: &mut BinaryReader,
    compressed: bool,
) -> Option<(packets::opcodes::Pkt, Vec<u8>)> {
    let service_uuid = reader.read_u64().ok()?;
    // read and ignore stub id (4 bytes)
    let _ = reader.read_u32().ok()?;
    let method_id_raw = reader.read_u32().ok()?;

    if service_uuid != 0x0000000063335342 {
        debug!("Notify: service_uuid mismatch: {service_uuid:x}");
        return None;
    }

    let msg_payload = reader.read_remaining().to_vec();
    if compressed {
        match zstd::decode_all(msg_payload.as_slice()) {
            Ok(decoded) => Some((Pkt::try_from(method_id_raw).ok()?, decoded)),
            Err(e) => {
                debug!("Notify: zstd decompression failed: {e}");
                None
            }
        }
    } else {
        Some((Pkt::try_from(method_id_raw).ok()?, msg_payload))
    }
}

/// Convenience to decompress bytes if needed (kept for potential reuse).
pub fn maybe_decompress(payload: Vec<u8>, compressed: bool) -> Option<Vec<u8>> {
    if compressed {
        zstd::decode_all(payload.as_slice())
            .map_err(|e| {
                debug!("zstd decompression failed: {e}");
            })
            .ok()
    } else {
        Some(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::opcodes::Pkt;

    fn u64_to_be_bytes(v: u64) -> [u8; 8] {
        v.to_be_bytes()
    }

    fn u32_to_be_bytes(v: u32) -> [u8; 4] {
        v.to_be_bytes()
    }

    #[test]
    fn parse_notify_uncompressed() {
        // Build reader containing: service_uuid(u64), stub_id(u32), method_id(u32), payload
        let service_uuid = 0x0000000063335342u64;
        let stub_id = 0x11223344u32;
        let method_id = 0x00000006u32; // Pkt::SyncNearEntities
        let payload = b"hello-world".to_vec();

        let mut data = Vec::new();
        data.extend_from_slice(&u64_to_be_bytes(service_uuid));
        data.extend_from_slice(&u32_to_be_bytes(stub_id));
        data.extend_from_slice(&u32_to_be_bytes(method_id));
        data.extend_from_slice(&payload);

        let mut reader = BinaryReader::from(data);
        let result = parse_notify_fragment(&mut reader, false).expect("should parse");
        assert_eq!(result.0, Pkt::SyncNearEntities);
        assert_eq!(result.1, b"hello-world".to_vec());
    }

    #[test]
    fn parse_notify_compressed() {
        let service_uuid = 0x0000000063335342u64;
        let stub_id = 0x55667788u32;
        let method_id = 0x00000006u32; // Pkt::SyncNearEntities
        let payload = b"the quick brown fox jumps over the lazy dog".to_vec();

        // compress payload
        let compressed = zstd::encode_all(payload.as_slice(), 0).expect("compress");

        let mut data = Vec::new();
        data.extend_from_slice(&u64_to_be_bytes(service_uuid));
        data.extend_from_slice(&u32_to_be_bytes(stub_id));
        data.extend_from_slice(&u32_to_be_bytes(method_id));
        data.extend_from_slice(&compressed);

        let mut reader = BinaryReader::from(data);
        let result = parse_notify_fragment(&mut reader, true).expect("should parse and decompress");
        assert_eq!(result.0, Pkt::SyncNearEntities);
        assert_eq!(
            result.1,
            b"the quick brown fox jumps over the lazy dog".to_vec()
        );
    }
}
