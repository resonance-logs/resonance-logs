use crate::packets::market::{MarketListing, MarketListingsBatch};
use bytes::Bytes;
use log::{debug, info};
use marketprotobuf_lib::marketproto;
use prost::Message;

/// Attempt to decode market listings from a `FrameDown` decompressed payload.
///
/// The market frames appear to contain a protobuf-ish stream where we look for
/// repeated occurrences of field 1 (key `0x0A`) which contain
/// `ExchangeNoticeDetailReply` messages.
pub fn decode_exchange_notice_detail(
    nested: &[u8],
    server_sequence: u32,
) -> Option<MarketListingsBatch> {
    debug!(target: "app::market", "decode_exchange_notice_detail: input len={}", nested.len());
    let mut listings: Vec<MarketListing> = Vec::new();
    let mut candidate_count = 0;

    for payload in iter_len_delimited_field_one_messages(nested) {
        candidate_count += 1;
        debug!(target: "app::market", "  Candidate #{}: payload len={}", candidate_count, payload.len());

        // The iterator extracts content AFTER the 0x0A tag and length varint.
        // The 0x0A tag represents field 1 (wire type LEN), so the payload is
        // the content of field 1 - which is ExchangeNoticeDetailReply directly.
        let reply = match marketproto::ExchangeNoticeDetailReply::decode(Bytes::copy_from_slice(
            payload,
        )) {
            Ok(r) => r,
            Err(e) => {
                debug!(target: "app::market", "  Candidate #{}: ExchangeNoticeDetailReply decode failed: {}", candidate_count, e);
                continue;
            }
        };

        if reply.items.is_empty() {
            debug!(target: "app::market", "  Candidate #{}: items is empty", candidate_count);
            continue;
        }

        // Sanity check: prost's decode is very lenient and will "succeed" on garbage data.
        // Real market data should have reasonable values for total_page and cur_page.
        // total_page is typically 1-1000, cur_page is typically 1-total_page.
        if reply.total_page < 0 || reply.total_page > 100_000 {
            debug!(target: "app::market", "  Candidate #{}: total_page={} out of reasonable range, skipping", candidate_count, reply.total_page);
            continue;
        }
        if reply.cur_page < 0 || reply.cur_page > reply.total_page.max(1) {
            debug!(target: "app::market", "  Candidate #{}: cur_page={} out of reasonable range (total_page={}), skipping", candidate_count, reply.cur_page, reply.total_page);
            continue;
        }

        // Additional sanity check: items should have reasonable prices (not negative, not absurdly high)
        // and should have item_info populated for real market listings.
        let valid_items_count = reply
            .items
            .iter()
            .filter(|entry| {
                entry.price >= 0
                    && entry.price < 1_000_000_000
                    && entry.num > 0
                    && entry.num < 1_000_000_000
                    && entry.item_info.is_some()
            })
            .count();

        if valid_items_count == 0 {
            debug!(target: "app::market", "  Candidate #{}: no items have valid item_info, skipping as likely garbage", candidate_count);
            continue;
        }
        info!(target: "app::market", "  Candidate #{}: {} items found", candidate_count, reply.items.len());
        info!(target: "app::market", "  Reply: total_page={}, cur_page={}, min_price={}, err_code={:?}",
            reply.total_page, reply.cur_page, reply.min_price, reply.err_code);

        for (i, entry) in reply.items.iter().enumerate() {
            // Log raw entry fields
            info!(target: "app::market", "    Entry[{}]: price={}, num={}, guid={:?}, notice_time={}",
                i, entry.price, entry.num, entry.guid, entry.notice_time);
            if let Some(ref item) = entry.item_info {
                info!(target: "app::market", "    Entry[{}].item_info: uuid={}, config_id={}, count={}",
                    i, item.uuid, item.config_id, item.count);
            } else {
                info!(target: "app::market", "    Entry[{}].item_info: None", i);
            }
        }

        for entry in reply.items {
            let item_config_id = entry.item_info.as_ref().and_then(|it| {
                if it.config_id != 0 {
                    Some(it.config_id)
                } else {
                    None
                }
            });

            // Print decoded market listing to console
            println!(
                "[Market] Item: {:?} | Price: {} Luno | Qty: {} | GUID: {} | NoticeTime: {:?}",
                item_config_id,
                entry.price,
                entry.num,
                if entry.guid.is_empty() {
                    "<none>".to_string()
                } else {
                    entry.guid.clone()
                },
                if entry.notice_time == 0 {
                    None
                } else {
                    Some(entry.notice_time)
                }
            );

            listings.push(MarketListing {
                server_sequence,
                price_luno: entry.price,
                quantity: entry.num,
                item_config_id,
                guid: if entry.guid.is_empty() {
                    None
                } else {
                    Some(entry.guid)
                },
                notice_time: if entry.notice_time == 0 {
                    None
                } else {
                    Some(entry.notice_time)
                },
            });
        }
    }

    debug!(target: "app::market", "decode_exchange_notice_detail: candidates={}, listings={}", candidate_count, listings.len());
    if listings.is_empty() {
        None
    } else {
        println!(
            "[Market] Batch decoded: {} listings (server_seq={})",
            listings.len(),
            server_sequence
        );
        Some(MarketListingsBatch {
            server_sequence,
            listings,
        })
    }
}

fn iter_len_delimited_field_one_messages<'a>(data: &'a [u8]) -> impl Iterator<Item = &'a [u8]> {
    MarketPayloadIter { data, idx: 0 }
}

struct MarketPayloadIter<'a> {
    data: &'a [u8],
    idx: usize,
}

impl<'a> Iterator for MarketPayloadIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.data.len() {
            if self.data[self.idx] != 0x0A {
                self.idx += 1;
                continue;
            }

            let mut slice = &self.data[(self.idx + 1)..];
            let Ok(len_u64) = prost::encoding::decode_varint(&mut slice) else {
                self.idx += 1;
                continue;
            };
            let len = usize::try_from(len_u64).ok()?;
            let header_len = self.data[(self.idx + 1)..].len() - slice.len();
            let payload_start = self.idx + 1 + header_len;
            let payload_end = payload_start.saturating_add(len);

            if payload_end > self.data.len() {
                // Stop: declared message length runs past the buffer.
                debug!(target: "app::market", "Market payload length overrun (start={}, len={}, buflen={})", payload_start, len, self.data.len());
                self.idx = self.data.len();
                return None;
            }

            self.idx = payload_end;
            return Some(&self.data[payload_start..payload_end]);
        }
        None
    }
}

/// Heuristic check for whether `bytes` looks like a nested framed packet stream.
///
/// resonance-logs expects framed packets to start with a big-endian u32 length.
pub fn looks_like_framed_packet_stream(bytes: &[u8]) -> bool {
    if bytes.len() < 6 {
        return false;
    }
    let len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    if len < 6 || len > bytes.len() {
        return false;
    }

    // Next 2 bytes after the u32 length are a big-endian u16 type.
    // We accept anything but reject obvious garbage like all-zeros packet type with tiny length.
    let pkt_type = u16::from_be_bytes([bytes[4], bytes[5]]);
    !(pkt_type == 0 && len < 10)
}
