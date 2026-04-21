use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

use memmap2::MmapMut;
use sha1::{Digest, Sha1};
use tokio::sync::mpsc;

use crate::metainfo::info_dictionary::InfoDictionary;
use crate::peer::worker::BLOCK_SIZE;
use crate::piece::PieceManager;

pub enum DiskEvent {
    PieceVerified(u32),
    PieceFailed(u32),
}

pub struct Block {
    pub index: u32,
    pub begin: u32,
    pub data: Vec<u8>,
}

struct Progress {
    received: HashSet<u32>,
    total_blocks: u32,
}

impl Progress {
    fn new(total_blocks: u32) -> Self {
        Self { received: HashSet::with_capacity(total_blocks as usize), total_blocks }
    }

    fn insert(&mut self, begin: u32) {
        self.received.insert(begin);
    }

    fn is_complete(&self) -> bool {
        self.received.len() == self.total_blocks as usize
    }
}

pub struct DiskManager {
    mmap: MmapMut,
    info_dictionary: Arc<InfoDictionary>,
    progress: HashMap<u32, Progress>,
    disk_event_tx: mpsc::Sender<DiskEvent>,
}

impl DiskManager {
    pub fn new(
        path: &Path,
        length: u64,
        info_dictionary: Arc<InfoDictionary>,
        disk_event_tx: mpsc::Sender<DiskEvent>,
    ) -> anyhow::Result<Self> {
        let file = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(path)?;

        file.set_len(length)?;

        Ok(Self { mmap: unsafe { MmapMut::map_mut(&file)? }, info_dictionary, progress: HashMap::new(), disk_event_tx })
    }

    pub async fn resume(&mut self, piece_manager: &mut PieceManager) {
        let count = self.info_dictionary.piece_count();

        for i in 0..count {
            let offset = i * self.info_dictionary.piece_length as usize;
            let len = self.info_dictionary.piece_len(i as u32) as usize;

            let hash: [u8; 20] = Sha1::digest(&self.mmap[offset..offset + len]).into();
            if self.info_dictionary.piece_hash(i) == Some(hash) {
                tracing::debug!("[DiskManager] Resuming: Piece {i}/{count}");
                piece_manager.mark_have(i as u32);
            }
        }
    }

    pub async fn run(mut self, mut block_rx: mpsc::Receiver<Block>) {
        while let Some(block) = block_rx.recv().await {
            let piece_index = block.index;

            if let Err(err) = self.write_block(block).await {
                tracing::error!("[DiskManager]: Write error on Piece {piece_index}: {err}");
                continue;
            }

            if self.progress.get(&piece_index).map(|p| p.is_complete()).unwrap_or(false) {
                self.progress.remove(&piece_index);
                self.verify_hash(piece_index);
            }
        }
    }

    async fn write_block(&mut self, block: Block) -> anyhow::Result<()> {
        let offset = block.index as usize * self.info_dictionary.piece_length as usize + block.begin as usize;
        let end = offset + block.data.len();

        self.mmap[offset..end].copy_from_slice(&block.data);

        let piece_len = self.info_dictionary.piece_len(block.index);
        let total_blocks = piece_len.div_ceil(BLOCK_SIZE as u64);

        self.progress.entry(block.index).or_insert_with(|| Progress::new(total_blocks as u32)).insert(block.begin);

        Ok(())
    }

    fn verify_hash(&self, piece_index: u32) {
        let offset = piece_index as usize * self.info_dictionary.piece_length as usize;
        let len = self.info_dictionary.piece_len(piece_index) as usize;

        let data = &self.mmap[offset..offset + len];
        let expected = self.info_dictionary.piece_hash(piece_index as usize);

        let got: [u8; 20] = Sha1::digest(data).into();
        let event = if Some(got) == expected {
            DiskEvent::PieceVerified(piece_index)
        } else {
            tracing::error!("[DiskManager]: Piece {piece_index} SHA1 mismatch");
            DiskEvent::PieceFailed(piece_index)
        };

        if self.disk_event_tx.try_send(event).is_err() {
            tracing::error!("[DiskManager]: channel full");
        }
    }
}
