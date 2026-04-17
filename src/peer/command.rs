pub enum PeerCommand {
    Request { index: u32, begin: u32, length: u32 }
}
