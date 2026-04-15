const BLOCK_SIZE: u16 = 16 * 1024;

pub(crate) struct InProgress {
    blocks: Vec<Option<Vec<u8>>>,
    requested: Vec<bool>,
    piece_length: u64,
    block_count: usize,
}

impl InProgress {
    pub(crate) fn new(piece_length: u64) -> Self {
        let block_count = piece_length.div_ceil(BLOCK_SIZE as u64) as usize;

        Self { blocks: vec![None; block_count], requested: vec![false; block_count], piece_length, block_count }
    }

    pub(crate) fn cancel_unfinished(&mut self) {
        for (index, request) in self.requested.iter_mut().enumerate() {
            if self.blocks[index].is_none() {
                *request = false
            }
        }
    }
}
