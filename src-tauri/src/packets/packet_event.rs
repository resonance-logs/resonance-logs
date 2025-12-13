use crate::packets::market::MarketListingsBatch;
use crate::packets::opcodes::Pkt;

/// Output events produced by the capture + reassembly + fragment processing pipeline.
#[derive(Debug)]
pub enum PacketEvent {
    /// A standard Notify fragment: (opcode, protobuf payload bytes).
    Notify { op: Pkt, data: Vec<u8> },

    /// Market listings extracted from a FrameDown payload.
    MarketListings(MarketListingsBatch),
}
