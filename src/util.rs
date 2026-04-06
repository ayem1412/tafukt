use rand::RngExt;
use rand::distr::Alphanumeric;

const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn generate_peer_id() -> [u8; 20] {
    let mut rng = rand::rng();
    let mut peer_id = [0u8; 20];

    peer_id[0] = b'-';
    peer_id[1] = b'T';
    peer_id[2] = b'F';

    let version = env!("CARGO_PKG_VERSION");
    let version_bytes = version_to_bytes(version);
    peer_id[3..7].copy_from_slice(&version_bytes);

    peer_id[7] = b'-';

    for i in 8..20 {
        peer_id[i] = rng.sample(Alphanumeric) as u8;
    }

    peer_id
}

fn version_to_bytes(version: &str) -> [u8; 4] {
    let parts: Vec<&str> = version.split('.').collect();
    [
        parts.get(0).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0),
        parts.get(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0),
        parts.get(2).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0),
        parts.get(3).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0),
    ]
}
