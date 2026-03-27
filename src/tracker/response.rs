use std::net::Ipv4Addr;

/// Tracker responses are bencoded dictionaries.
#[derive(Debug)]
pub struct TrackerSuccessResponse {
    /// Maps to the number of seconds the downloader should wait between regular rerequests.
    interval: u64,

    /// Maps to a list of dictionaries corresponding to peers, each of which contains the keys peer
    /// id, ip, and port, which map to the peer's self-selected ID, IP address or dns name as a
    /// string, and port number, respectively.
    // (IP, PORT)
    pub peers: Vec<(Ipv4Addr, u16)>,
}

impl TrackerSuccessResponse {
    pub fn new(interval: u64, peers: Vec<(Ipv4Addr, u16)>) -> Self {
        Self { interval, peers }
    }
}
