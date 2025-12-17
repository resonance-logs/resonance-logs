//! Market data uploader module.
//!
//! Sends market listings to server

use crate::packets::market::MarketListingsBatch;
use log::{debug, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use tauri::AppHandle;

use super::{get_module_sync_settings_path, read_module_sync_settings_blocking};

/// Base URL for the market API (uses the same server as encounter uploads).
const MARKET_API_URL: &str = "https://api.bpsr.app/api/v1/market/upload";

/// Lazily loaded item name map.
static ITEM_NAME_MAP: OnceLock<HashMap<i32, String>> = OnceLock::new();

/// Request payload for uploading market data.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MarketListing {
    price: i64,
    quantity: i32,
    seller_guid: Option<String>,
    notice_time: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MarketUploadRequest {
    item_id: u32,
    item_name: String,
    listings: Vec<MarketListing>,
}

/// Response from the market upload endpoint.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MarketUploadResponse {
    status: String,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    retry_after_sec: Option<i32>,
}

/// Load the item name map from the embedded JSON file.
fn load_item_name_map() -> HashMap<i32, String> {
    // The item_name_map.json is located in src-tauri/meter-data/
    // Path is relative to this source file: src/uploader/market_uploader.rs
    let json_str = include_str!("../../meter-data/item_name_map.json");
    match serde_json::from_str::<HashMap<String, String>>(json_str) {
        Ok(raw_map) => {
            // Convert string keys to i32
            raw_map
                .into_iter()
                .filter_map(|(k, v)| k.parse::<i32>().ok().map(|id| (id, v)))
                .collect()
        }
        Err(e) => {
            warn!("Failed to parse item_name_map.json: {}", e);
            HashMap::new()
        }
    }
}

/// Get the item name map, loading it if necessary.
fn get_item_name_map() -> &'static HashMap<i32, String> {
    ITEM_NAME_MAP.get_or_init(load_item_name_map)
}

/// Check if an item ID is tracked in the item name map.
pub fn is_tracked_item(item_id: i32) -> bool {
    get_item_name_map().contains_key(&item_id)
}

/// Get the name of an item by its ID.
pub fn get_item_name(item_id: i32) -> Option<&'static str> {
    get_item_name_map().get(&item_id).map(|s| s.as_str())
}

/// Check if market uploading is enabled (API key exists in settings).
/// This is a blocking operation - call from blocking context or spawn_blocking.
fn has_api_key(app: &AppHandle) -> bool {
    let Some(settings_path) = get_module_sync_settings_path(app) else {
        return false;
    };

    let Some(settings) = read_module_sync_settings_blocking(settings_path) else {
        return false;
    };

    !settings.api_key.trim().is_empty()
}

/// Upload a batch of market listings to the server.
///
/// Only uploads if the user has an API key set.
/// Only listings with item IDs present in the item name map will be uploaded.
/// The server handles throttling (15-minute intervals per item).
pub async fn upload_market_listings(app: AppHandle, batch: &MarketListingsBatch) {
    // Check if user has API key set (run in blocking context)
    let app_check = app.clone();
    let has_key = tokio::task::spawn_blocking(move || has_api_key(&app_check))
        .await
        .unwrap_or(false);

    if !has_key {
        debug!("[Market Uploader] Skipping upload - no API key set");
        return;
    }

    let item_map = get_item_name_map();
    let client = Client::new();

    // Group listings by item ID
    let mut item_listings: HashMap<i32, Vec<MarketListing>> = HashMap::new();

    for listing in &batch.listings {
        let Some(item_id) = listing.item_config_id else {
            continue;
        };

        // Only process items that are in our tracking list
        if item_map.contains_key(&item_id) {
            item_listings
                .entry(item_id)
                .or_default()
                .push(MarketListing {
                    price: listing.price_luno as i64,
                    quantity: listing.quantity,
                    seller_guid: listing.guid.clone(),
                    notice_time: listing.notice_time,
                });
        }
    }

    // debug!("[Market Uploader] Processing batch with {} unique tracked items", item_listings.len());

    let mut uploaded_count = 0;
    let mut throttled_count = 0;
    let mut error_count = 0;

    for (item_id, listings) in item_listings {
        let Some(item_name) = item_map.get(&item_id) else {
            // Should be covered by the check above, but safe to check again
            continue;
        };

        let request = MarketUploadRequest {
            item_id: item_id as u32,
            item_name: item_name.clone(),
            listings,
        };

        match client.post(MARKET_API_URL).json(&request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<MarketUploadResponse>().await {
                        Ok(resp) => {
                            if resp.status == "throttled" {
                                throttled_count += 1;
                                debug!(
                                    "[Market Uploader] Item {} throttled (retry in {:?}s)",
                                    item_id, resp.retry_after_sec
                                );
                            } else {
                                uploaded_count += 1;
                                debug!("[Market Uploader] Item {} uploaded successfully", item_id);
                            }
                        }
                        Err(e) => {
                            error_count += 1;
                            warn!(
                                "[Market Uploader] Failed to parse response for item {}: {}",
                                item_id, e
                            );
                        }
                    }
                } else {
                    error_count += 1;
                    warn!(
                        "[Market Uploader] Server error for item {}: {}",
                        item_id,
                        response.status()
                    );
                }
            }
            Err(e) => {
                error_count += 1;
                warn!(
                    "[Market Uploader] Request failed for item {}: {}",
                    item_id, e
                );
            }
        }
    }

    if uploaded_count > 0 || throttled_count > 0 || error_count > 0 {
        info!(
            "[Market Uploader] Batch complete: {} items uploaded, {} throttled, {} errors",
            uploaded_count, throttled_count, error_count
        );
    }
}
