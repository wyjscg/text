use std::collections::HashMap;

pub struct IntSet {
    members: HashMap<i32, i32>,
}

impl IntSet {
    pub fn new() -> Self {
        IntSet {
            members: HashMap::new(),
        }
    }

    pub fn has(&self, i: i32) -> bool {
        self.members.get(&i).map_or(false, |&count| count > 0)
    }

    pub fn reset(&mut self) {
        self.members.clear();
    }

    pub fn increment(&mut self, i: i32) {
        *self.members.entry(i).or_insert(0) += 1;
    }

    pub fn decrement(&mut self, i: i32) {
        if let Some(count) = self.members.get_mut(&i) {
            if *count <= 1 {
                self.members.remove(&i);
            } else {
                *count -= 1;
            }
        }
    }
}

impl Default for IntSet {
    fn default() -> Self {
        Self::new()
    }
}
