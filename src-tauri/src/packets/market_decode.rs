use crate::packets::market::{MarketListing, MarketListingsBatch};
use bytes::Bytes;
use log::{debug, info};
use marketprotobuf_lib::marketproto;
use prost::Message;

/// Attempt to decode market listings from a `FrameDown` decompressed payload.
///
/// The market frames appear to contain a protobuf-ish stream where we look for
/// repeated occurrences of field 1 (key `0x0A`) which contain
/// exchange-related `*_Ret` messages.
pub fn decode_exchange_notice_detail(
    nested: &[u8],
    server_sequence: u32,
) -> Option<MarketListingsBatch> {
    debug!(target: "app::market", "decode_exchange_notice_detail: input len={}", nested.len());
    let mut listings: Vec<MarketListing> = Vec::new();
    let mut candidate_count = 0;
    let mut decoded_any = false;

    // Decode helpers: some captures contain the `*_Ret` wrapper message, while others contain
    // only the embedded reply message (i.e. bytes of field 1 of the wrapper). We support both.
    fn add_notice_detail_reply(
        server_sequence: u32,
        reply: marketproto::ExchangeNoticeDetailReply,
        listings: &mut Vec<MarketListing>,
    ) -> bool {
        if reply.items.is_empty()
            || reply.total_page < 0
            || reply.total_page > 100_000
            || reply.cur_page < 0
            || reply.cur_page > reply.total_page.max(1)
        {
            return false;
        }

        // Additional sanity check: at least one entry looks like a real listing.
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
            return false;
        }

        info!(
            target: "app::market",
            "  ExchangeNoticeDetailReply {} items (server_seq={})",
            reply.items.len(),
            server_sequence
        );
        info!(
            target: "app::market",
            "  Reply: total_page={}, cur_page={}, min_price={}, err_code={:?}",
            reply.total_page,
            reply.cur_page,
            reply.min_price,
            reply.err_code
        );

        for entry in reply.items {
            let item_config_id = entry.item_info.as_ref().and_then(|it| {
                if it.config_id != 0 { Some(it.config_id) } else { None }
            });

            println!(
                "[Market] Item: {:?} | Price: {} Luno | Qty: {} | GUID: {} | NoticeTime: {:?}",
                item_config_id,
                entry.price,
                entry.num,
                if entry.guid.is_empty() { "<none>".to_string() } else { entry.guid.clone() },
                if entry.notice_time == 0 { None } else { Some(entry.notice_time) }
            );

            listings.push(MarketListing {
                server_sequence,
                price_luno: entry.price,
                quantity: entry.num,
                item_config_id,
                guid: if entry.guid.is_empty() { None } else { Some(entry.guid) },
                notice_time: if entry.notice_time == 0 { None } else { Some(entry.notice_time) },
            });
        }

        true
    }

    fn add_notice_reply(
        server_sequence: u32,
        reply: marketproto::ExchangeNoticeReply,
        listings: &mut Vec<MarketListing>,
    ) -> bool {
        if reply.items.is_empty() {
            return false;
        }

        let original_items_len = reply.items.len();

        fn looks_like_unix_timestamp_seconds(value: u32) -> bool {
            // Rough heuristic: 2020-01-01..2035-01-01.
            (1_577_836_800..=2_050_000_000).contains(&value)
        }

        fn is_plausible_item_config_id(config_id: u32) -> bool {
            // In practice item config IDs are "game data" identifiers (e.g. 1083004).
            // We reject very small IDs and values that look like epoch timestamps.
            if config_id < 1_000 {
                return false;
            }
            if looks_like_unix_timestamp_seconds(config_id) {
                return false;
            }
            // Guardrail against nonsense: if this ever blocks a real item, loosen it.
            config_id <= 100_000_000
        }

        fn is_plausible_quantity(num: u32) -> bool {
            // Market quantities can be large, but multi-million counts are almost certainly
            // a false-positive decode (we've seen these come from timestamp-ish payloads).
            (1..=1_000_000).contains(&num)
        }

        fn is_plausible_min_price_luno(min_price: u32) -> bool {
            // We only log entries with a non-zero min price.
            // Zero is a strong signal that we've decoded the wrong message.
            (1..=1_000_000_000).contains(&min_price)
        }

        let valid_entries: Vec<marketproto::ExchangeItemInfo> = reply
            .items
            .into_iter()
            .filter(|entry| {
                is_plausible_item_config_id(entry.config_id)
                    && is_plausible_quantity(entry.num)
                    && is_plausible_min_price_luno(entry.min_price)
            })
            .collect();

        if valid_entries.is_empty() {
            // We decoded something protobuf-shaped, but it does not look like a market item list.
            // This is common because we scan for nested length-delimited blobs and protobuf
            // decoding is permissive.
            debug!(
                target: "app::market",
                "  ExchangeNoticeReply rejected by heuristics (server_seq={})",
                server_sequence
            );
            return false;
        }

        info!(
            target: "app::market",
            "  ExchangeNoticeReply {} items (valid={}, server_seq={})",
            original_items_len,
            valid_entries.len(),
            server_sequence
        );

        for entry in valid_entries {
            let item_config_id = i32::try_from(entry.config_id).ok();
            let price_luno = i32::try_from(entry.min_price).unwrap_or(i32::MAX);
            let quantity = i32::try_from(entry.num).unwrap_or(i32::MAX);

            println!(
                "[Market] Item: {:?} | MinPrice: {} Luno | Qty: {}",
                item_config_id, price_luno, quantity
            );

            listings.push(MarketListing {
                server_sequence,
                price_luno,
                quantity,
                item_config_id,
                guid: None,
                notice_time: None,
            });
        }

        true
    }

    fn add_lowest_price_reply(
        server_sequence: u32,
        reply: marketproto::ExchangeLowestPriceReply,
        listings: &mut Vec<MarketListing>,
    ) -> bool {
        if reply.config_id == 0 || reply.lowest_price <= 0 {
            return false;
        }

        let item_config_id = Some(reply.config_id);
        let price_luno = i32::try_from(reply.lowest_price).unwrap_or(i32::MAX);
        println!(
            "[Market] LowestPrice Item: {:?} | LowestPrice: {} Luno",
            item_config_id, price_luno
        );
        listings.push(MarketListing {
            server_sequence,
            price_luno,
            quantity: 1,
            item_config_id,
            guid: None,
            notice_time: None,
        });
        true
    }

    fn add_prebuy_reply(
        server_sequence: u32,
        reply: marketproto::ExchangeNoticePreBuyReply,
        listings: &mut Vec<MarketListing>,
    ) -> bool {
        if reply.items.is_empty() {
            return false;
        }

        // PreBuy replies use the same item entry shape as detail listings, but without paging.
        // Because protobuf decoding is permissive and we scan nested blobs, we must sanity-check
        // that at least one entry looks like a real market listing.
        let valid_items: Vec<marketproto::ExchangePriceItemData> = reply
            .items
            .into_iter()
            .filter(|entry| {
                let Some(item_info) = entry.item_info.as_ref() else {
                    return false;
                };
                if item_info.config_id == 0 {
                    return false;
                }

                // Typical values: config_id ~ 1_000_000, price in luno, quantity in 1..hundreds.
                (1..=1_000_000_000).contains(&entry.price)
                    && (1..=1_000_000).contains(&entry.num)
                    && (1..=100_000_000).contains(&item_info.config_id)
            })
            .collect();

        if valid_items.is_empty() {
            debug!(
                target: "app::market",
                "  ExchangeNoticePreBuyReply rejected by heuristics (server_seq={})",
                server_sequence
            );
            return false;
        }

        info!(
            target: "app::market",
            "  ExchangeNoticePreBuyReply {} items (valid={}, server_seq={})",
            valid_items.len(),
            valid_items.len(),
            server_sequence
        );

        for entry in valid_items {
            let item_config_id = entry
                .item_info
                .as_ref()
                .and_then(|it| if it.config_id != 0 { Some(it.config_id) } else { None });
            println!(
                "[Market] PreBuy Item: {:?} | Price: {} Luno | Qty: {}",
                item_config_id, entry.price, entry.num
            );
            listings.push(MarketListing {
                server_sequence,
                price_luno: entry.price,
                quantity: entry.num,
                item_config_id,
                guid: if entry.guid.is_empty() { None } else { Some(entry.guid) },
                notice_time: if entry.notice_time == 0 {
                    None
                } else {
                    Some(entry.notice_time)
                },
            });
        }

        true
    }

    for payload in iter_len_delimited_field_one_messages(nested) {
        candidate_count += 1;
        debug!(target: "app::market", "  Candidate #{}: payload len={}", candidate_count, payload.len());

        // The iterator extracts content AFTER the 0x0A tag and length varint.
        // In practice we see multiple exchange-related `*_Ret` wrappers here depending
        // on what the market UI is doing.

        // 1) Try detailed listings: wrapper and raw reply variants.
        if let Ok(ret_msg) = marketproto::ExchangeNoticeDetailRet::decode(Bytes::copy_from_slice(payload)) {
            if let Some(reply) = ret_msg.ret {
                // Even if we reject it by heuristics (empty items, etc.), decoding succeeded.
                decoded_any = true;
                if add_notice_detail_reply(server_sequence, reply, &mut listings) {
                    continue;
                }
            }
        }
        if let Ok(reply) = marketproto::ExchangeNoticeDetailReply::decode(Bytes::copy_from_slice(payload)) {
            decoded_any = true;
            if add_notice_detail_reply(server_sequence, reply, &mut listings) {
                continue;
            }
        }

        // 2) Try high-level item list: wrapper and raw reply variants.
        // This is commonly sent when opening/browsing the market without drilling into an item.
        if let Ok(ret_msg) = marketproto::ExchangeNoticeRet::decode(Bytes::copy_from_slice(payload)) {
            let Some(reply) = ret_msg.ret else {
                continue;
            };
            decoded_any = true;
            if add_notice_reply(server_sequence, reply, &mut listings) {
                continue;
            }
        }

        if let Ok(reply) = marketproto::ExchangeNoticeReply::decode(Bytes::copy_from_slice(payload)) {
            decoded_any = true;
            if add_notice_reply(server_sequence, reply, &mut listings) {
                continue;
            }
        }

        // 3) Try list/care list variants (wire-compatible but separated for clarity)
        if let Ok(ret_msg) = marketproto::ExchangeListRet::decode(Bytes::copy_from_slice(payload)) {
            if let Some(reply) = ret_msg.ret {
                decoded_any = true;
                if add_notice_reply(
                    server_sequence,
                    marketproto::ExchangeNoticeReply {
                        items: reply.items,
                        err_code: reply.err_code,
                    },
                    &mut listings,
                ) {
                    continue;
                }
            }
        }

        if let Ok(reply) = marketproto::ExchangeListReply::decode(Bytes::copy_from_slice(payload)) {
            decoded_any = true;
            if add_notice_reply(
                server_sequence,
                marketproto::ExchangeNoticeReply {
                    items: reply.items,
                    err_code: reply.err_code,
                },
                &mut listings,
            ) {
                continue;
            }
        }

        if let Ok(ret_msg) = marketproto::ExchangeCareListRet::decode(Bytes::copy_from_slice(payload)) {
            if let Some(reply) = ret_msg.ret {
                decoded_any = true;
                if add_notice_reply(
                    server_sequence,
                    marketproto::ExchangeNoticeReply {
                        items: reply.items,
                        err_code: reply.err_code,
                    },
                    &mut listings,
                ) {
                    continue;
                }
            }
        }

        if let Ok(reply) = marketproto::ExchangeCareListReply::decode(Bytes::copy_from_slice(payload)) {
            decoded_any = true;
            if add_notice_reply(
                server_sequence,
                marketproto::ExchangeNoticeReply {
                    items: reply.items,
                    err_code: reply.err_code,
                },
                &mut listings,
            ) {
                continue;
            }
        }

        // 4) Try pre-buy (uses listing-style entries but without paging fields)
        if let Ok(ret_msg) = marketproto::ExchangeNoticePreBuyRet::decode(Bytes::copy_from_slice(payload)) {
            if let Some(reply) = ret_msg.ret {
                decoded_any = true;
                if add_prebuy_reply(server_sequence, reply, &mut listings) {
                    continue;
                }
            }
        }

        if let Ok(reply) = marketproto::ExchangeNoticePreBuyReply::decode(Bytes::copy_from_slice(payload)) {
            decoded_any = true;
            if add_prebuy_reply(server_sequence, reply, &mut listings) {
                continue;
            }
        }

        // 5) Lowest price response (single-item)
        if let Ok(ret_msg) = marketproto::ExchangeLowestPriceRet::decode(Bytes::copy_from_slice(payload)) {
            if let Some(reply) = ret_msg.ret {
                decoded_any = true;
                if add_lowest_price_reply(server_sequence, reply, &mut listings) {
                    continue;
                }
            }
        }

        if let Ok(reply) = marketproto::ExchangeLowestPriceReply::decode(Bytes::copy_from_slice(payload)) {
            decoded_any = true;
            if add_lowest_price_reply(server_sequence, reply, &mut listings) {
                continue;
            }
        }

        // 6) Sale data (does not map cleanly to MarketListing, but we still recognize it to
        // reduce "unknown reply" noise on big frames).
        if let Ok(ret_msg) = marketproto::ExchangeSaleDataRet::decode(Bytes::copy_from_slice(payload)) {
            if let Some(reply) = ret_msg.ret {
                decoded_any = true;
                info!(
                    target: "app::market",
                    "  Candidate #{}: ExchangeSaleDataReply items={} min_rate={} err_code={:?}",
                    candidate_count,
                    reply.items.len(),
                    reply.min_rate,
                    reply.err_code
                );
            }
        }

        if let Ok(reply) = marketproto::ExchangeSaleDataReply::decode(Bytes::copy_from_slice(payload)) {
            decoded_any = true;
            info!(
                target: "app::market",
                "  Candidate #{}: ExchangeSaleDataReply items={} min_rate={} err_code={:?} (raw)",
                candidate_count,
                reply.items.len(),
                reply.min_rate,
                reply.err_code
            );
        }

        // Fallback: real traffic often nests useful market payloads one level deeper.
        // Scan this candidate for embedded field-1 length-delimited blobs and try decoding them
        // as reply messages.
        for embedded in iter_len_delimited_field_one_messages(payload) {
            // Most commonly this is the `ret` field bytes (i.e. the reply message).
            if let Ok(reply) = marketproto::ExchangeNoticeDetailReply::decode(Bytes::copy_from_slice(embedded)) {
                decoded_any = true;
                if add_notice_detail_reply(server_sequence, reply, &mut listings) {
                    break;
                }
            }
            if let Ok(reply) = marketproto::ExchangeNoticeReply::decode(Bytes::copy_from_slice(embedded)) {
                decoded_any = true;
                if add_notice_reply(server_sequence, reply, &mut listings) {
                    break;
                }
            }
            if let Ok(reply) = marketproto::ExchangeListReply::decode(Bytes::copy_from_slice(embedded)) {
                // Wire-identical to ExchangeNoticeReply.
                decoded_any = true;
                if add_notice_reply(server_sequence, marketproto::ExchangeNoticeReply {
                    items: reply.items,
                    err_code: reply.err_code,
                }, &mut listings) {
                    break;
                }
            }
            if let Ok(reply) = marketproto::ExchangeCareListReply::decode(Bytes::copy_from_slice(embedded)) {
                // Wire-identical to ExchangeNoticeReply.
                decoded_any = true;
                if add_notice_reply(server_sequence, marketproto::ExchangeNoticeReply {
                    items: reply.items,
                    err_code: reply.err_code,
                }, &mut listings) {
                    break;
                }
            }
            if let Ok(reply) = marketproto::ExchangeNoticePreBuyReply::decode(Bytes::copy_from_slice(embedded)) {
                decoded_any = true;
                if add_prebuy_reply(server_sequence, reply, &mut listings) {
                    break;
                }
            }
            if let Ok(reply) = marketproto::ExchangeLowestPriceReply::decode(Bytes::copy_from_slice(embedded)) {
                decoded_any = true;
                if add_lowest_price_reply(server_sequence, reply, &mut listings) {
                    break;
                }
            }

            if let Ok(reply) = marketproto::ExchangeSaleDataReply::decode(Bytes::copy_from_slice(embedded)) {
                decoded_any = true;
                info!(
                    target: "app::market",
                    "  ExchangeSaleDataReply items={} min_rate={} err_code={:?} (embedded)",
                    reply.items.len(),
                    reply.min_rate,
                    reply.err_code
                );
                // Keep scanning embedded payloads; sale data doesn't add listings.
            }
        }
    }

    debug!(target: "app::market", "decode_exchange_notice_detail: candidates={}, listings={}", candidate_count, listings.len());
    if !decoded_any {
        // Helps distinguish "we saw market-like framing" from "we recognized a concrete reply".
        let preview_len = nested.len().min(2048);
        let count_0a = nested[..preview_len].iter().filter(|&&b| b == 0x0A).count();
        if count_0a >= 10 {
            println!(
                "[Market] No known exchange replies decoded (server_seq={}, nested_len={}, 0x0A(first{}B)={}).",
                server_sequence,
                nested.len(),
                preview_len,
                count_0a
            );
        }
    }
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
            let len = match usize::try_from(len_u64) {
                Ok(v) => v,
                Err(_) => {
                    // Likely a stray 0x0A inside some other payload. Keep scanning.
                    self.idx += 1;
                    continue;
                }
            };
            let header_len = self.data[(self.idx + 1)..].len() - slice.len();
            let payload_start = self.idx + 1 + header_len;
            let payload_end = payload_start.saturating_add(len);

            if payload_end > self.data.len() {
                // Declared message length runs past the buffer. This is common when we hit a
                // 0x0A byte that is not actually a field-1 length-delimited message.
                debug!(target: "app::market", "Market payload length overrun (start={}, len={}, buflen={})", payload_start, len, self.data.len());
                self.idx += 1;
                continue;
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

#[cfg(test)]
mod tests {
    use super::decode_exchange_notice_detail;
    use marketprotobuf_lib::marketproto;
    use prost::Message;

    fn encode_varint(mut value: u64) -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                break;
            }
        }
        out
    }

    #[test]
    fn decodes_wrapped_exchange_notice_detail_ret() {
        let reply = marketproto::ExchangeNoticeDetailReply {
            items: vec![marketproto::ExchangePriceItemData {
                price: 123,
                num: 2,
                item_info: Some(marketproto::ItemLite {
                    uuid: 1,
                    config_id: 456,
                    count: 2,
                }),
                guid: "abc".to_string(),
                notice_time: 999,
            }],
            total_page: 1,
            cur_page: 1,
            min_price: 0,
            err_code: 0,
        };

        let ret = marketproto::ExchangeNoticeDetailRet { ret: Some(reply) };
        let payload = ret.encode_to_vec();

        // Nested buffer format scanned by the decoder:
        //   0x0A + varint length + payload
        let mut nested = vec![0x0A];
        nested.extend_from_slice(&encode_varint(payload.len() as u64));
        nested.extend_from_slice(&payload);

        let batch = decode_exchange_notice_detail(&nested, 42)
            .expect("expected decoder to extract listings from wrapped payload");
        assert_eq!(batch.server_sequence, 42);
        assert_eq!(batch.listings.len(), 1);

        let listing = &batch.listings[0];
        assert_eq!(listing.server_sequence, 42);
        assert_eq!(listing.price_luno, 123);
        assert_eq!(listing.quantity, 2);
        assert_eq!(listing.item_config_id, Some(456));
        assert_eq!(listing.guid.as_deref(), Some("abc"));
        assert_eq!(listing.notice_time, Some(999));
    }

    #[test]
    fn decodes_wrapped_exchange_notice_ret() {
        let reply = marketproto::ExchangeNoticeReply {
            items: vec![marketproto::ExchangeItemInfo {
                config_id: 1083004,
                num: 5,
                min_price: 1234,
                is_care: false,
            }],
            err_code: 0,
        };

        let ret = marketproto::ExchangeNoticeRet { ret: Some(reply) };
        let payload = ret.encode_to_vec();

        let mut nested = vec![0x0A];
        nested.extend_from_slice(&encode_varint(payload.len() as u64));
        nested.extend_from_slice(&payload);

        let batch = decode_exchange_notice_detail(&nested, 7)
            .expect("expected decoder to extract items from wrapped notice payload");
        assert_eq!(batch.server_sequence, 7);
        assert_eq!(batch.listings.len(), 1);

        let listing = &batch.listings[0];
        assert_eq!(listing.server_sequence, 7);
        assert_eq!(listing.item_config_id, Some(1083004));
        assert_eq!(listing.quantity, 5);
        assert_eq!(listing.price_luno, 1234);
        assert!(listing.guid.is_none());
        assert!(listing.notice_time.is_none());
    }

    #[test]
    fn rejects_notice_reply_with_all_zero_min_price() {
        let reply = marketproto::ExchangeNoticeReply {
            items: vec![marketproto::ExchangeItemInfo {
                config_id: 1083004,
                num: 551,
                min_price: 0,
                is_care: false,
            }],
            err_code: 0,
        };

        let ret = marketproto::ExchangeNoticeRet { ret: Some(reply) };
        let payload = ret.encode_to_vec();

        let mut nested = vec![0x0A];
        nested.extend_from_slice(&encode_varint(payload.len() as u64));
        nested.extend_from_slice(&payload);

        let batch = decode_exchange_notice_detail(&nested, 99);
        assert!(
            batch.is_none(),
            "expected decoder to reject zero-price notice replies to avoid false positives"
        );
    }

    #[test]
    fn rejects_notice_reply_that_looks_like_timestamps() {
        let reply = marketproto::ExchangeNoticeReply {
            items: vec![marketproto::ExchangeItemInfo {
                config_id: 1_764_936_000,
                num: 1_768_460_400,
                min_price: 1,
                is_care: false,
            }],
            err_code: 0,
        };

        let ret = marketproto::ExchangeNoticeRet { ret: Some(reply) };
        let payload = ret.encode_to_vec();

        let mut nested = vec![0x0A];
        nested.extend_from_slice(&encode_varint(payload.len() as u64));
        nested.extend_from_slice(&payload);

        let batch = decode_exchange_notice_detail(&nested, 100);
        assert!(
            batch.is_none(),
            "expected decoder to reject timestamp-looking config IDs"
        );
    }
}
