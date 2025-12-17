//! Standalone test for market decoding from a binary capture file.
//!
//! Run with: cargo test --package resonance-logs --lib -- packets::market_decode_test::test_decode_tc1_bin --nocapture

#[cfg(test)]
mod tests {
    use crate::packets::market_decode::decode_exchange_notice_detail;

    // Frame iteration logic to extract FrameDown bodies
    fn iter_frames(data: &[u8]) -> impl Iterator<Item = (usize, u16, bool, &[u8])> {
        struct FrameIter<'a> {
            data: &'a [u8],
            offset: usize,
        }

        impl<'a> Iterator for FrameIter<'a> {
            type Item = (usize, u16, bool, &'a [u8]);

            fn next(&mut self) -> Option<Self::Item> {
                while self.offset + 6 <= self.data.len() {
                    let length = u32::from_be_bytes([
                        self.data[self.offset],
                        self.data[self.offset + 1],
                        self.data[self.offset + 2],
                        self.data[self.offset + 3],
                    ]) as usize;

                    if length == 0 || self.offset + length > self.data.len() {
                        self.offset += 1;
                        continue;
                    }

                    let pkt_type = u16::from_be_bytes([
                        self.data[self.offset + 4],
                        self.data[self.offset + 5],
                    ]);

                    let is_zstd = (pkt_type & 0x8000) != 0;
                    let fragment_type = pkt_type & 0x7FFF;
                    let body = &self.data[(self.offset + 6)..(self.offset + length)];
                    let current_offset = self.offset;
                    self.offset += length;

                    return Some((current_offset, fragment_type, is_zstd, body));
                }
                None
            }
        }

        FrameIter { data, offset: 0 }
    }

    #[test]
    fn test_decode_tc1_bin() {
        // Load the test file
        let data = match std::fs::read("test_market_data.bin") {
            Ok(d) => d,
            Err(e) => {
                println!("Could not read test_market_data.bin: {}", e);
                println!("Place test market data capture file at src-tauri/test_market_data.bin");
                return;
            }
        };

        println!("Data len: {}", data.len());

        let mut total_listings = 0;
        let mut framedown_count = 0;

        for (offset, fragment_type, is_zstd, body) in iter_frames(&data) {
            if fragment_type != 0x0006 {
                // Only process FrameDown packets
                continue;
            }
            framedown_count += 1;

            if body.len() < 4 {
                println!(
                    "FrameDown @0x{:06x}: body too short ({})",
                    offset,
                    body.len()
                );
                continue;
            }

            let server_seq = u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
            let nested = &body[4..];

            // Decompress if zstd
            let decompressed: Vec<u8>;
            let nested_bytes = if is_zstd {
                match zstd::decode_all(nested) {
                    Ok(v) => {
                        println!(
                            "FrameDown @0x{:06x}: zstd {} -> {} bytes",
                            offset,
                            nested.len(),
                            v.len()
                        );
                        decompressed = v;
                        &decompressed[..]
                    }
                    Err(e) => {
                        println!("FrameDown @0x{:06x}: zstd fail: {}", offset, e);
                        continue;
                    }
                }
            } else {
                nested
            };

            println!(
                "FrameDown @0x{:06x}: server_seq={}, nested_len={}",
                offset,
                server_seq,
                nested_bytes.len()
            );

            // Print first 64 bytes as hex for debugging
            if nested_bytes.len() > 10 {
                let preview: String = nested_bytes[..std::cmp::min(64, nested_bytes.len())]
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("  Hex preview: {}", preview);
            }

            // Try to decode market data
            if let Some(batch) = decode_exchange_notice_detail(nested_bytes, server_seq) {
                println!("  SUCCESS: {} listings", batch.listings.len());
                for listing in &batch.listings {
                    println!(
                        "    Price: {}, ConfigID: {:?}, Qty: {}",
                        listing.price_luno, listing.item_config_id, listing.quantity
                    );
                }
                total_listings += batch.listings.len();
            } else {
                println!("  No listings decoded");
            }
        }

        println!("\n=== Summary ===");
        println!("FrameDown packets: {}", framedown_count);
        println!("Total listings: {}", total_listings);

        // Now specifically check the two known market data packets
        println!("\n=== Checking specific packets ===");
        for (offset, fragment_type, is_zstd, body) in iter_frames(&data) {
            if fragment_type != 0x0006 || body.len() < 4 {
                continue;
            }
            let server_seq = u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
            if server_seq != 538 && server_seq != 780 {
                continue;
            }

            let nested = &body[4..];
            let decompressed: Vec<u8>;
            let nested_bytes = if is_zstd {
                decompressed = zstd::decode_all(nested).unwrap();
                &decompressed[..]
            } else {
                nested
            };

            println!(
                "\nFrameDown @0x{:06x}: server_seq={}, nested_len={}, is_zstd={}",
                offset,
                server_seq,
                nested_bytes.len(),
                is_zstd
            );

            // Print first 128 bytes as hex
            let preview: String = nested_bytes[..std::cmp::min(128, nested_bytes.len())]
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            println!("  Hex: {}", preview);

            // Count 0x0A occurrences
            let count_0a = nested_bytes.iter().filter(|&&b| b == 0x0A).count();
            println!("  0x0A count: {}", count_0a);

            // Find positions of 0x0A
            let positions: Vec<usize> = nested_bytes
                .iter()
                .enumerate()
                .filter(|&(_, &b)| b == 0x0A)
                .map(|(i, _)| i)
                .take(10)
                .collect();
            println!("  First 10 0x0A positions: {:?}", positions);

            // Try decode
            if let Some(batch) = decode_exchange_notice_detail(nested_bytes, server_seq) {
                println!("  DECODE SUCCESS: {} listings", batch.listings.len());
            } else {
                println!("  DECODE FAILED");
            }
        }
    }
}
