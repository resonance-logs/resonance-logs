use crate::packets::market::{MarketListing, MarketListingsBatch};
use bytes::Bytes;
use log::debug;
use marketprotobuf_lib::marketproto;
use prost::Message;

/// Attempt to decode market listings from a `FrameDown` decompressed payload.
///
/// The market frames appear to contain a protobuf-ish stream where we look for
/// repeated occurrences of field 1 (key `0x0A`) and treat each as a standalone
/// `ExchangeNoticeDetailRet` message.
pub fn decode_exchange_notice_detail(
    nested: &[u8],
    server_sequence: u32,
) -> Option<MarketListingsBatch> {
    let mut listings: Vec<MarketListing> = Vec::new();

    for payload in iter_len_delimited_field_one_messages(nested) {
        let Ok(ret) = marketproto::ExchangeNoticeDetailRet::decode(Bytes::copy_from_slice(payload))
        else {
            continue;
        };

        let Some(reply) = ret.ret else {
            continue;
        };
        if reply.items.is_empty() {
            continue;
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
