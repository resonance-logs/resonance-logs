use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) struct Server {
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
        write!(f, "{}:{} -> {}:{}", ip_to_str(&self.src_addr), self.src_port, ip_to_str(&self.dst_addr), self.dst_port)
    }
}

fn ip_to_str(ip: &[u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

