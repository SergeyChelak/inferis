use std::mem::{self};

type Representation = u128;

pub struct Footprint {
    raw: u128,
}

impl Footprint {
    pub fn new() -> Self {
        Self { raw: 0 }
    }

    pub fn set(&mut self, pos: usize, value: bool) {
        let val = 1 << pos;
        if value {
            self.raw |= val;
        } else {
            self.raw &= !val;
        }
    }

    pub fn _get(&self, pos: usize) -> bool {
        self.raw & 1 << pos > 0
    }

    pub fn is_matches(&self, other: &Self) -> bool {
        self.raw & other.raw == self.raw
    }

    pub fn max_items() -> usize {
        mem::size_of::<Representation>()
    }
}
