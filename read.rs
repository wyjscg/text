impl Sparse {
    pub fn insert(&mut self, x: i64) -> bool {
        let (offset, i) = offset_and_bit_index(x);

        let mut b = self.first();
        while !std::ptr::eq(b, unsafe { &NONE }) && b.offset <= offset {
            if b.offset == offset {
                let b_mut = unsafe { &mut *(b as *const Block as *mut Block) };
                return b_mut.insert(i);
            }
            b = self.next(b);
        }

        // Insert new block before b.
        let new = self.insert_block_before(b as *const Block);
        let new_mut = unsafe { &mut *new };
        new_mut.offset = offset;
        new_mut.insert(i)
    }

    fn insert_block_before(&mut self, next: *const Block) -> *mut Block {
        if self.is_empty() {
            if !std::ptr::eq(next, unsafe { &NONE as *const Block }) {
                panic!("BUG: passed block with empty set");
            }
            return &mut self.root as *mut Block;
        }

        let root_ptr = &mut self.root as *mut Block;
        let none_ptr = unsafe { &NONE as *const Block };

        if std::ptr::eq(next, root_ptr as *const Block) {
            // Special case: create a new block that will become the root block.
            // The old root block becomes the second block.
            let second = Box::new(self.root.clone());
            let second_ptr = Box::into_raw(second);
            
            unsafe {
                let second_ref = &mut *second_ptr;
                let old_next = second_ref.next;
                
                self.root.next = Some(second_ptr);
                
                if std::ptr::eq(old_next.unwrap(), root_ptr) {
                    self.root.prev = Some(second_ptr);
                } else {
                    self.root.prev = second_ref.prev;
                    if let Some(next_ptr) = second_ref.next {
                        (*next_ptr).prev = Some(second_ptr);
                    }
                    second_ref.prev = Some(root_ptr);
                }
            }
            
            return root_ptr;
        }

        let next_ptr = if std::ptr::eq(next, none_ptr) {
            root_ptr
        } else {
            next as *mut Block
        };

        unsafe {
            let next_ref = &mut *next_ptr;
            let b = Box::new(Block {
                offset: 0,
                bits: [0; WORDS_PER_BLOCK],
                next: Some(next_ptr),
                prev: next_ref.prev,
            });
            let b_ptr = Box::into_raw(b);
            
            if let Some(prev_ptr) = next_ref.prev {
                (*prev_ptr).next = Some(b_ptr);
            }
            next_ref.prev = Some(b_ptr);
            
            b_ptr
        }
    }
}

impl Block {
    fn insert(&mut self, i: usize) -> bool {
        let (w, mask) = word_mask(i);
        if self.bits[w] & mask == 0 {
            self.bits[w] |= mask;
            return true;
        }
        false
    }
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Block {
            offset: self.offset,
            bits: self.bits,
            next: self.next,
            prev: self.prev,
        }
    }
}
