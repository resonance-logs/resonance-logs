use std::collections::BTreeMap;
use etherparse::TcpSlice;

pub struct TCPReassembler {
    cache: BTreeMap<u32, Vec<u8>>, // sequence -> payload
    next_seq: Option<u32>,          // next expected sequence
}

impl TCPReassembler {
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
            next_seq: None,
        }
    }

    // Push a TCP segment and try to reassemble contiguous data.
    /// Returns Some(Vec<u8>) if contiguous data is available, None otherwise.
    pub fn push_segment(&mut self, packet: TcpSlice) -> Option<(u32, Vec<u8>)> {
        let payload = packet.payload().to_vec();
        let seq = packet.sequence_number();
        if payload.is_empty() {
            return None;
        }

        // Insert segment into cache
        self.cache.insert(seq, payload);

        // Initialize next_seq to the lowest sequence seen if not set
        if self.next_seq.is_none() {
            if let Some((&lowest_seq, _)) = self.cache.first_key_value() {
                self.next_seq = Some(lowest_seq);
            }
        }

        // Try to assemble contiguous data
        let mut output = Vec::new();
        while let Some(next) = self.next_seq {
            if let Some(segment) = self.cache.remove(&next) {
                // advance next_seq only when we actually use this segment
                self.next_seq = Some(next.wrapping_add(segment.len() as u32));
                output.extend(segment);
            } else {
                break;
            }
        }

        if output.is_empty() {
            None
        } else {
            Some((self.next_seq?, output))
        }
    }

    pub fn clear_reassembler(&mut self, seq_number: u32) {
        self.cache = BTreeMap::new();
        self.next_seq = Some(seq_number)
    }
}
