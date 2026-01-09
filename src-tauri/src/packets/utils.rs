use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::{fmt, io};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Server {
    src_addr: [u8; 4],
    src_port: u16,
    dst_addr: [u8; 4],
    dst_port: u16,
}

impl Server {
    pub fn new(src_addr: [u8; 4], src_port: u16, dst_addr: [u8; 4], dst_port: u16) -> Self {
        Self {
            src_addr,
            src_port,
            dst_addr,
            dst_port,
        }
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} -> {}:{}",
            ip_to_str(&self.src_addr),
            self.src_port,
            ip_to_str(&self.dst_addr),
            self.dst_port
        )
    }
}

fn ip_to_str(ip: &[u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

#[inline]
pub fn tcp_sequence_before(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) < 0
}

#[inline]
pub fn tcp_sequence_after(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b) as i32) > 0
}

pub struct TCPReassembler {
    cache: BTreeMap<u32, Vec<u8>>, // sequence -> payload
    next_seq: Option<u32>,         // next expected sequence
    buffered_bytes: usize,         // Total bytes currently in the cache
}

const MAX_TCP_CACHE_SIZE: usize = 5 * 1024 * 1024; // 5MB limit

impl TCPReassembler {
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
            next_seq: None,
            buffered_bytes: 0,
        }
    }

    /// Insert a TCP payload segment for the given sequence number.
    /// Returns Some(Vec<u8>) when new contiguous bytes starting at the
    /// expected sequence become available.
    pub fn insert_segment(&mut self, sequence_number: u32, payload: &[u8]) -> Option<Vec<u8>> {
        if payload.is_empty() {
            return None;
        }

        let expected = match self.next_seq {
            Some(seq) => seq,
            None => {
                self.next_seq = Some(sequence_number);
                sequence_number
            }
        };

        let mut start_seq = sequence_number;
        let mut data = payload;

        if tcp_sequence_before(start_seq, expected) {
            let overlap = expected.wrapping_sub(start_seq) as usize;
            if overlap >= data.len() {
                return None;
            }
            start_seq = expected;
            data = &data[overlap..];
        }

        // Avoid storing duplicates unless the new payload is longer.
        match self.cache.get_mut(&start_seq) {
            Some(existing) => {
                if data.len() > existing.len() {
                    self.buffered_bytes -= existing.len();
                    existing.clear();
                    existing.extend_from_slice(data);
                    self.buffered_bytes += existing.len();
                }
            }
            None => {
                self.cache.insert(start_seq, data.to_vec());
                self.buffered_bytes += data.len();
            }
        }

        // Gap Skipping / Buffer Limit Enforcement
        if self.buffered_bytes > MAX_TCP_CACHE_SIZE {
            log::warn!(
                "TCPReassembler buffer exceeded limit ({} bytes). Dropping cached segments and resyncing at {}",
                self.buffered_bytes,
                start_seq
            );

            self.cache.clear();
            self.buffered_bytes = 0;
            self.next_seq = Some(start_seq.wrapping_add(data.len() as u32));
            return Some(data.to_vec());
        }

        let mut cursor = self.next_seq.unwrap();
        let mut output: Vec<u8> = Vec::new();

        while let Some(mut segment) = self.cache.remove(&cursor) {
            self.buffered_bytes -= segment.len();
            cursor = cursor.wrapping_add(segment.len() as u32);
            if output.is_empty() {
                output = std::mem::take(&mut segment);
            } else {
                output.extend_from_slice(&segment);
            }
        }

        if output.is_empty() {
            None
        } else {
            self.next_seq = Some(cursor);
            Some(output)
        }
    }

    pub fn reset(&mut self, next_seq: Option<u32>) {
        self.cache.clear();
        self.buffered_bytes = 0;
        self.next_seq = next_seq;
    }

    pub fn next_sequence(&self) -> Option<u32> {
        self.next_seq
    }
}

// Binary reader implementation
pub struct BinaryReader {
    pub cursor: Cursor<Vec<u8>>,
}

impl BinaryReader {
    pub fn from(data: Vec<u8>) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        self.cursor.read_u16::<BigEndian>()
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.cursor.read_u32::<BigEndian>()
    }

    pub fn peek_u32(&mut self) -> io::Result<u32> {
        let pos = self.cursor.position();
        let value = self.cursor.read_u32::<BigEndian>()?;
        self.cursor.set_position(pos);
        Ok(value)
    }

    pub fn read_u64(&mut self) -> io::Result<u64> {
        self.cursor.read_u64::<BigEndian>()
    }

    pub fn read_string(&mut self) -> io::Result<String> {
        let mut s = String::new();
        self.cursor.read_to_string(&mut s)?;
        Ok(s)
    }

    pub fn read_bytes(&mut self, count: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; count];
        self.cursor.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn read_remaining(&mut self) -> &[u8] {
        let pos = self.cursor.position() as usize;
        let buf = self.cursor.get_ref();
        &buf[pos..]
    }

    pub fn remaining(&self) -> usize {
        let total_len = self.cursor.get_ref().len() as u64;
        let current_pos = self.cursor.position();
        (total_len.saturating_sub(current_pos)) as usize
    }

    pub fn len(&self) -> usize {
        self.cursor.get_ref().len()
    }

    pub fn splice_remaining(&mut self, data: &[u8]) {
        let start_range = self.cursor.position() as usize;
        let buf = self.cursor.get_mut();
        buf.splice(start_range.., data.iter().cloned());
    }
}

#[cfg(test)]
mod tests {
    use super::TCPReassembler;

    #[test]
    fn reassembles_in_order() {
        let mut reassembler = TCPReassembler::new();
        assert_eq!(
            reassembler.insert_segment(10, b"abc"),
            Some(b"abc".to_vec())
        );
        assert_eq!(
            reassembler.insert_segment(13, b"def"),
            Some(b"def".to_vec())
        );
    }

    #[test]
    fn reassembles_out_of_order_once_gap_filled() {
        let mut reassembler = TCPReassembler::new();
        assert_eq!(
            reassembler.insert_segment(100, b"abc"),
            Some(b"abc".to_vec())
        );
        assert!(reassembler.insert_segment(106, b"ghi").is_none());
        assert_eq!(
            reassembler.insert_segment(103, b"def"),
            Some(b"defghi".to_vec())
        );
    }

    #[test]
    fn trims_overlapping_segments_and_ignores_duplicates() {
        let mut reassembler = TCPReassembler::new();
        assert!(reassembler.insert_segment(50, b"abc").is_some());
        // Duplicate shorter payload should be ignored
        assert!(reassembler.insert_segment(50, b"ab").is_none());
        // Overlapping payload should emit only unseen bytes
        assert_eq!(
            reassembler.insert_segment(51, b"bcdef"),
            Some(b"def".to_vec())
        );
    }

    #[test]
    fn reset_drops_state_and_reinitializes() {
        let mut reassembler = TCPReassembler::new();
        assert!(reassembler.insert_segment(500, b"abc").is_some());
        reassembler.reset(None);
        assert_eq!(reassembler.next_sequence(), None);
        assert_eq!(
            reassembler.insert_segment(42, b"xyz"),
            Some(b"xyz".to_vec())
        );
    }

    #[test]
    fn overflow_drops_cache_and_resyncs() {
        // Build up >5MiB of buffered out-of-order data to trigger overflow.
        // Use moderately sized segments so the test doesn't allocate a single giant Vec.
        const CHUNK: usize = 1024 * 1024; // 1 MiB

        let mut r = TCPReassembler::new();
        // Establish an expected sequence that we will not satisfy.
        assert!(r.insert_segment(10, b"a").is_some());
        let expected = r.next_sequence().unwrap();

        // Insert out-of-order chunks far ahead so they accumulate in cache.
        // The final insert should trigger overflow and return its payload immediately.
        let mut last_seq: u32 = expected.wrapping_add(10_000);
        for i in 0..6 {
            let payload = vec![i as u8; CHUNK];
            let out = r.insert_segment(last_seq, &payload);
            if i < 5 {
                assert!(out.is_none());
            } else {
                assert_eq!(out.as_ref().map(|v| v.len()), Some(CHUNK));
                assert_eq!(out.unwrap(), payload);
                assert_eq!(r.next_sequence(), Some(last_seq.wrapping_add(CHUNK as u32)));
            }
            last_seq = last_seq.wrapping_add((CHUNK as u32).wrapping_add(123));
        }
    }
}
