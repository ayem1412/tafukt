use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::{Arc, Mutex};

use memmap2::MmapMut;
use sha1::{Digest, Sha1};

use crate::metainfo::info_dictionary::InfoDictionary;
use crate::peer::manager::BLOCK_SIZE;
use crate::piece::PieceManager;

pub struct Block {
    pub index: u32,
    pub begin: u32,
    pub data: Vec<u8>,
}

pub struct DiskManager {
    mmap: MmapMut,
    piece_length: u64,
    piece_manager: Arc<Mutex<PieceManager>>,
    info: InfoDictionary,
    blocks_remaining: HashMap<u32, u32>,
}

impl DiskManager {
    pub fn new(
        path: &Path,
        length: u64,
        piece_length: u64,
        piece_manager: Arc<Mutex<PieceManager>>,
        info: InfoDictionary,
    ) -> anyhow::Result<Self> {
        let file = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(path)?;

        file.set_len(length)?;

        Ok(Self {
            mmap: unsafe { MmapMut::map_mut(&file)? },
            piece_length,
            piece_manager,
            info,
            blocks_remaining: HashMap::new(),
        })
    }

    pub async fn handle_block(&mut self, block: Block) -> anyhow::Result<()> {
        if self.piece_manager.lock().unwrap().is_complete() {
            return Ok(());
        }

        let offset = block.index as usize * self.piece_length as usize + block.begin as usize;
        let end = offset + block.data.len();

        self.mmap[offset..end].copy_from_slice(&block.data);

        let piece_len = self.info.piece_len(block.index);
        let total_blocks = piece_len.div_ceil(BLOCK_SIZE as u64);

        let remaining = self.blocks_remaining.entry(block.index).or_insert(total_blocks as u32);
        *remaining -= 1;

        if *remaining == 0 {
            self.blocks_remaining.remove(&block.index);
            self.verify_hash(block.index).await?;
        }

        Ok(())
    }

    async fn verify_hash(&self, piece_index: u32) -> anyhow::Result<()> {
        let offset = piece_index as usize * self.piece_length as usize;
        let len = self.info.piece_len(piece_index) as usize;

        let data = self.mmap[offset..offset + len].to_vec();
        let expected = self.info.piece_hash(piece_index as usize).expect("out of range `piece_index`");

        let ok = tokio::task::spawn_blocking(move || {
            let got: [u8; 20] = Sha1::digest(&data).into();
            got == expected
        })
        .await
        .unwrap_or(false);

        if !ok {
            tracing::error!("DiskManager: Piece {piece_index} SHA1 mismatch - releasing for retry");
            self.piece_manager.lock().unwrap().release(piece_index);
            return Ok(());
        }

        if let Err(err) = self.mmap.flush_range(offset, self.piece_length as usize) {
            tracing::error!("[Disk]: flush error on piece {}: {err}", piece_index);
        }

        let mut piece_manager = self.piece_manager.lock().unwrap();
        piece_manager.mark_have(piece_index);

        tracing::info!(
            "DiskManager: Piece {piece_index} verified - {}/{} ({:.1}%)",
            piece_manager.have.count_ones(),
            piece_manager.have.piece_count,
            piece_manager.have.count_ones() as f64 / piece_manager.have.piece_count as f64 * 100.0
        );

        Ok(())
    }
}
