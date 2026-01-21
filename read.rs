use std::mem;

pub struct Sparse {
    root: Block,
}

type Word = usize;

const BITS_PER_WORD: usize = mem::size_of::<Word>() * 8;
const BITS_PER_BLOCK: usize = 256;
const WORDS_PER_BLOCK: usize = BITS_PER_BLOCK / BITS_PER_WORD;

struct Block {
    offset: i64,
    bits: [Word; WORDS_PER_BLOCK],
    next: Option<*mut Block>,
    prev: Option<*mut Block>,
}

static mut NONE: Block = Block {
    offset: 0,
    bits: [0; WORDS_PER_BLOCK],
    next: None,
    prev: None,
};

impl Sparse {
    pub fn len(&self) -> usize {
        let mut l = 0;
        let mut b = self.first();
        while !std::ptr::eq(b, unsafe { &NONE }) {
            l += b.len();
            b = self.next(b);
        }
        l
    }

    pub fn is_empty(&self) -> bool {
        self.root.next.is_none() || self.root.offset == i64::MAX
    }

    pub fn take_min(&mut self, p: &mut i64) -> bool {
        if self.is_empty() {
            return false;
        }
        *p = self.root.min(true);
        if self.root.empty() {
            let root_ptr = &mut self.root as *mut Block;
            self.remove_block(root_ptr);
        }
        true
    }

    pub fn clear(&mut self) {
        let root_ptr = &mut self.root as *mut Block;
        self.root.offset = i64::MAX;
        self.root.next = Some(root_ptr);
        self.root.prev = Some(root_ptr);
    }

    fn first(&self) -> &Block {
        self.init();
        if self.root.offset == i64::MAX {
            unsafe { &NONE }
        } else {
            &self.root
        }
    }

    fn init(&self) {
        let root = unsafe { &mut *((&self.root) as *const Block as *mut Block) };
        if root.next.is_none() {
            root.offset = i64::MAX;
            root.next = Some(root as *mut Block);
            root.prev = Some(root as *mut Block);
        } else if let Some(next_ptr) = root.next {
            let next = unsafe { &*next_ptr };
            if let Some(next_prev) = next.prev {
                if !std::ptr::eq(next_prev, root) {
                    panic!("to copy a Sparse you must call its Copy method");
                }
            }
        }
    }

    fn next(&self, b: &Block) -> &Block {
        if let Some(next_ptr) = b.next {
            if std::ptr::eq(next_ptr, &self.root as *const Block) {
                unsafe { &NONE }
            } else {
                unsafe { &*next_ptr }
            }
        } else {
            unsafe { &NONE }
        }
    }

    fn remove_block(&mut self, b: *mut Block) -> *const Block {
        let root_ptr = &mut self.root as *mut Block;
        
        if !std::ptr::eq(b, root_ptr) {
            unsafe {
                let block = &mut *b;
                let prev = &mut *block.prev.unwrap();
                let next = &mut *block.next.unwrap();
                
                prev.next = Some(next as *mut Block);
                next.prev = Some(prev as *mut Block);
                
                if std::ptr::eq(next as *const Block, root_ptr) {
                    return &NONE as *const Block;
                }
                return next as *const Block;
            }
        }

        let first_ptr = self.root.next.unwrap();
        
        if std::ptr::eq(first_ptr, root_ptr) {
            self.clear();
            return unsafe { &NONE as *const Block };
        }

        let first = unsafe { &mut *first_ptr };
        self.root.offset = first.offset;
        self.root.bits = first.bits;

        if let Some(first_next) = first.next {
            if std::ptr::eq(first_next, root_ptr) {
                self.root.next = Some(root_ptr);
                self.root.prev = Some(root_ptr);
            } else {
                self.root.next = Some(first_next);
                unsafe { (*first_next).prev = Some(root_ptr) };
            }
        }
        
        root_ptr as *const Block
    }
}

impl Block {
    fn len(&self) -> usize {
        let mut l = 0;
        for &w in &self.bits {
            l += popcount(w);
        }
        l
    }

    fn min(&mut self, take: bool) -> i64 {
        for i in 0..self.bits.len() {
            let w = self.bits[i];
            if w != 0 {
                let tz = ntz(w);
                if take {
                    self.bits[i] = w & !(1 << tz);
                }
                return self.offset + (i * BITS_PER_WORD) as i64 + tz as i64;
            }
        }
        panic!("BUG: empty block")
    }

    fn empty(&self) -> bool {
        for &w in &self.bits {
            if w != 0 {
                return false;
            }
        }
        true
    }
}

fn popcount(x: Word) -> usize {
    x.count_ones() as usize
}

fn ntz(x: Word) -> usize {
    x.trailing_zeros() as usize
}
