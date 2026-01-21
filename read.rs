impl Sparse {
    pub fn max(&self) -> i64 {
        if self.is_empty() {
            return i64::MIN;
        }
        unsafe {
            let prev_ptr = self.root.prev.unwrap();
            (*prev_ptr).max()
        }
    }

    pub fn has(&self, x: i64) -> bool {
        let (offset, i) = offset_and_bit_index(x);
        if let Some(b_ptr) = self.block(offset) {
            let b = unsafe { &*b_ptr };
            return b.has(i);
        }
        false
    }
}

impl Block {
    fn max(&self) -> i64 {
        let mut bi = self.offset + BITS_PER_BLOCK as i64;
        // Decrement bi by number of high zeros in last bits.
        for i in (0..self.bits.len()).rev() {
            let w = self.bits[i];
            if w != 0 {
                return bi - nlz(w) as i64 - 1;
            }
            bi -= BITS_PER_WORD as i64;
        }
        panic!("BUG: empty block")
    }

    fn has(&self, i: usize) -> bool {
        let (w, mask) = word_mask(i);
        self.bits[w] & mask != 0
    }
}

fn nlz(x: Word) -> usize {
    x.leading_zeros() as usize
}
