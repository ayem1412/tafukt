use tokio::fs::File;
use tokio::io::{self, AsyncSeekExt, AsyncWriteExt};

#[derive(Debug)]
pub struct Piece {
    index: u32,
    begin: u32,
    block: Vec<u8>,
}

impl Piece {
    pub fn new(index: u32, begin: u32, block: Vec<u8>) -> Self {
        Self { index, begin, block }
    }

    pub async fn save_block_to_disk(&self, length: u64, name: &str) -> io::Result<()> {
        let offset = self.index as u64 * length + self.begin as u64;

        let mut file = File::options().create(true).truncate(false).write(true).open(name).await?;

        file.seek(std::io::SeekFrom::Start(offset)).await?;
        file.write_all(&self.block).await?;
        file.flush().await?;

        Ok(())
    }
}
