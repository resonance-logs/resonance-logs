use crate::packets;

pub fn start() { // todo: add app_handle?
    // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
    // 1. Start capturing packets
    // 2. Use the channel to receive packets back and process them
    let _rx = packets::packet_capture::start_capture(); // Since live meter is not critical, it's ok to just log it // TODO: maybe bubble an error up to the frontend instead?
    // while let Ok((op, data)) = rx.recv().await {
    //
    // }

}
