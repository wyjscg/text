impl Sparse {
    pub fn remove(&mut self, x: i64) -> bool {
        let (offset, i) = offset_and_bit_index(x);
        if let Some(b_ptr) = self.block(offset) {
            let b = unsafe { &mut *b_ptr };
            if !b.remove(i) {
                return false;
            }
            if b.empty() {
                self.remove_block(b_ptr);
            }
            return true;
        }
        false
    }

    fn block(&self, offset: i64) -> Option<*mut Block> {
        let mut b = self.first();
        while !std::ptr::eq(b, unsafe { &NONE }) && b.offset <= offset {
            if b.offset == offset {
                return Some(b as *const Block as *mut Block);
            }
            b = self.next(b);
        }
        None
    }
}

impl Block {
    fn remove(&mut self, i: usize) -> bool {
        let (w, mask) = word_mask(i);
        if self.bits[w] & mask != 0 {
            self.bits[w] &= !mask;
            return true;
        }
        false
    }
}

fn offset_and_bit_index(x: i64) -> (i64, usize) {
    let mut mod_val = x % BITS_PER_BLOCK as i64;
    if mod_val < 0 {
        mod_val += BITS_PER_BLOCK as i64;
    }
    (x - mod_val, mod_val as usize)
}

fn word_mask(i: usize) -> (usize, Word) {
    let w = i / BITS_PER_WORD;
    let mask = 1 << (i % BITS_PER_WORD);
    (w, mask)
}
