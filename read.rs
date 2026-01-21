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
        self.len() == 0
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
}

impl Block {
    fn len(&self) -> usize {
        let mut l = 0;
        for &w in &self.bits {
            l += popcount(w);
        }
        l
    }
}

fn popcount(x: Word) -> usize {
    x.count_ones() as usize
}
