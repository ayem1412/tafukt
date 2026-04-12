use std::fs::OpenOptions;
use std::path::Path;

use memmap2::MmapMut;

pub struct Block {
    pub index: u32,
    pub begin: u32,
    pub data: Vec<u8>,
}

pub struct DiskManager {
    mmap: MmapMut,
    piece_length: u64,
}

impl DiskManager {
    pub fn new(path: &Path, length: u64, piece_length: u64) -> anyhow::Result<Self> {
        let file = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(path)?;

        file.set_len(length)?;

        Ok(Self { mmap: unsafe { MmapMut::map_mut(&file)? }, piece_length })
    }

    pub fn handle_block(&mut self, block: Block) {
        let offset = block.index as usize * self.piece_length as usize + block.begin as usize;
        let end = offset + block.data.len();

        self.mmap[offset..end].copy_from_slice(&block.data);

        if let Err(err) = self.mmap.flush_range(offset, self.piece_length as usize) {
            tracing::error!("[Disk]: flush error on piece {}: {err}", block.index);
        }
    }
}
