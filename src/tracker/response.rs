use std::net::SocketAddr;

/// Tracker responses are bencoded dictionaries.
#[derive(Debug)]
pub struct TrackerSuccessResponse {
    /// Maps to the number of seconds the downloader should wait between regular rerequests.
    pub interval: u64,

    /// Maps to a list of dictionaries corresponding to peers, each of which contains the keys peer
    /// id, ip, and port, which map to the peer's self-selected ID, IP address or dns name as a
    /// string, and port number, respectively.
    pub peers: Vec<SocketAddr>,
}

impl TrackerSuccessResponse {
    pub fn new(interval: u64, peers: Vec<SocketAddr>) -> Self {
        Self { interval, peers }
    }
}
