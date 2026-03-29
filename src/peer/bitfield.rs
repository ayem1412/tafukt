struct Bitfield([u8]);

impl Bitfield {
    fn has_piece(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let offset = index % 8;

        self.0[byte_index] >> (7 - offset) & 1 != 0
    }

    fn set_piece(&mut self, index: usize) {
        let byte_index = index / 8;
        let offset = index % 8;

        self.0[byte_index] |= 1 << (7 - offset)
    }
}
