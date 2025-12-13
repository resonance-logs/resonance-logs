use serde::Serialize;

/// A single Market listing entry extracted from live packets.
#[derive(Debug, Clone, Serialize)]
pub struct MarketListing {
    pub server_sequence: u32,
    pub price_luno: i32,
    pub quantity: i32,
    pub item_config_id: Option<i32>,
    pub guid: Option<String>,
    pub notice_time: Option<i64>,
}

/// Batch of listings extracted from one FrameDown payload.
#[derive(Debug, Clone, Serialize)]
pub struct MarketListingsBatch {
    pub server_sequence: u32,
    pub listings: Vec<MarketListing>,
}
