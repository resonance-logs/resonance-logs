use std::convert::TryInto;

/// A simple TCP reassembler for length-prefixed frames where each frame
/// starts with a u32 length (little-endian) followed by that many bytes.
///
/// The reassembler keeps a single Vec<u8> buffer and an offset cursor to avoid
/// cloning the whole buffer repeatedly. When a complete frame is available it
/// returns it as a Vec<u8> (one allocation per frame).
pub struct Reassembler {
    buffer: Vec<u8>,
    cursor: usize,
    /// Safety cap to avoid pathological allocations (can be tuned)
    max_buffer_size: usize,
}

impl Reassembler {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
            cursor: 0,
            max_buffer_size: 10 * 1024 * 1024, // 10 MB
        }
    }

    /// Push incoming bytes (e.g., TCP payload) into the reassembler.
    pub fn push(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
        // If buffer grows beyond max, we compact immediately to avoid OOM
        if self.buffer.len() > self.max_buffer_size {
            self.compact();
        }
    }

    /// Try to extract the next complete frame if available.
    /// Returns Some(frame_bytes) or None if not enough data yet.
    pub fn try_next(&mut self) -> Option<Vec<u8>> {
        // Need at least 4 bytes to read length
        if self.available_len() < 4 {
            return None;
        }

    // Read u32 big-endian from buffer[cursor..cursor+4]
    let len_bytes = &self.buffer[self.cursor..self.cursor + 4];
    let frame_len = u32::from_be_bytes(len_bytes.try_into().unwrap()) as usize;

        // Sanity check: frame length must be >= 4 (header included) and not absurd
        if frame_len == 0 || frame_len > self.max_buffer_size {
            // Avoid trying to parse insane frame sizes; drop buffer to recover.
            // Caller may decide to surface an error instead of silent recovery.
            self.cursor = self.buffer.len();
            self.compact();
            return None;
        }

        if self.available_len() < frame_len {
            // Not enough bytes yet
            return None;
        }

        let start = self.cursor;
        let end = self.cursor + frame_len;
        let frame = self.buffer[start..end].to_vec();

        self.cursor = end;
        // Compact buffer occasionally to discard consumed prefix
        if self.cursor > 4096 {
            self.compact();
        }

        Some(frame)
    }

    fn available_len(&self) -> usize {
        self.buffer.len().saturating_sub(self.cursor)
    }

    fn compact(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if self.cursor >= self.buffer.len() {
            self.buffer.clear();
            self.cursor = 0;
            return;
        }
        // Move remaining bytes to start
        let remaining = self.buffer.split_off(self.cursor);
        self.buffer = remaining;
        self.cursor = 0;
    }

    /// Feed an owned Vec<u8> into the reassembler without copying when possible.
    /// If the internal buffer is empty and cursor==0 we take ownership of the
    /// provided Vec to avoid an extra copy. Otherwise we extend the buffer.
    pub fn feed_owned(&mut self, bytes: Vec<u8>) {
        if self.cursor == 0 && self.buffer.is_empty() {
            // reuse the allocation
            self.buffer = bytes;
            return;
        }
        self.buffer.extend_from_slice(&bytes);
    }

    /// Take and return the remaining unconsumed bytes as a Vec<u8> and
    /// reset the internal buffer.
    pub fn take_remaining(&mut self) -> Vec<u8> {
        if self.cursor == 0 {
            let rem = std::mem::take(&mut self.buffer);
            return rem;
        }
        let rem = self.buffer.split_off(self.cursor);
        self.buffer = Vec::new();
        self.cursor = 0;
        rem
    }
}

#[cfg(test)]
mod tests {
    use super::Reassembler;

    fn make_frame(payload: &[u8]) -> Vec<u8> {
    let total_len = (4 + payload.len()) as u32;
    let mut v = total_len.to_be_bytes().to_vec();
        v.extend_from_slice(payload);
        v
    }

    #[test]
    fn single_frame_in_one_push() {
        let mut r = Reassembler::new();
        let frame = make_frame(b"hello");
        r.push(&frame);
        let got = r.try_next();
        assert!(got.is_some());
    assert_eq!(&got.unwrap()[4..], b"hello");
        assert!(r.try_next().is_none());
    }

    #[test]
    fn two_frames_in_one_push() {
        let mut r = Reassembler::new();
        let f1 = make_frame(b"foo");
        let f2 = make_frame(b"barbaz");
        let mut combined = Vec::new();
        combined.extend_from_slice(&f1);
        combined.extend_from_slice(&f2);
        r.push(&combined);
        let g1 = r.try_next().unwrap();
    assert_eq!(&g1[4..], b"foo");
        let g2 = r.try_next().unwrap();
    assert_eq!(&g2[4..], b"barbaz");
        assert!(r.try_next().is_none());
    }

    #[test]
    fn frame_split_across_pushes() {
        let mut r = Reassembler::new();
        let frame = make_frame(b"split-me");
        // push first half
        let split = frame.len() / 2;
        r.push(&frame[..split]);
        assert!(r.try_next().is_none());
        r.push(&frame[split..]);
        let got = r.try_next().unwrap();
    assert_eq!(&got[4..], b"split-me");
    }

    #[test]
    fn malformed_large_length_is_recovered() {
        let mut r = Reassembler::new();
        // put a ridiculously large length
        let huge = (r.max_buffer_size as u32 + 100).to_le_bytes();
        r.push(&huge);
        // no panic, and try_next returns None
        assert!(r.try_next().is_none());
        // after compaction, pushing a normal frame still succeeds
        let f = make_frame(b"ok");
        r.push(&f);
        let got = r.try_next().unwrap();
    assert_eq!(&got[4..], b"ok");
    }
}
