/// Generated market/trading protobuf bindings.
///
/// This crate is intentionally separate from `blueprotobuf-lib` so we can add
/// market-specific schemas without modifying the existing blueprotobuf module.
pub mod marketproto {
    include!(concat!(env!("OUT_DIR"), "/marketproto.rs"));
}
