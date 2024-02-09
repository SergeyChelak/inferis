use std::mem::size_of;

type Representation = u64;

#[derive(Default)]
pub struct Archetype {
    data: Representation,
}

impl Archetype {
    pub fn set(&mut self, position: usize, enabled: bool) {
        let val = 1 << position;
        if enabled {
            self.data |= val;
        } else {
            self.data &= !val;
        }
    }

    pub fn get(&self, position: usize) -> bool {
        self.data & 1 << position > 0
    }

    pub fn flip(&mut self, position: usize) {
        self.data ^= 1 << position;
    }

    pub fn add(&mut self, other: &Archetype) -> &mut Self {
        self.data |= other.data;
        self
    }

    pub fn remove(&mut self, other: &Archetype) -> &mut Self {
        self
    }

    pub fn matches(&self, other: &Archetype) -> bool {
        self.data & other.data == self.data
    }

    pub fn max_items() -> usize {
        8 * size_of::<Representation>()
    }
}

#[cfg(test)]
mod test {
    use crate::engine::archetype::Archetype;

    #[test]
    fn archetype_basic() {
        let mut val = Archetype::default();
        for pos in 0..Archetype::max_items() {
            assert!(!val.get(pos));
            val.set(pos, true);
            assert!(val.get(pos));
            val.set(pos, false);
            assert!(!val.get(pos));
            val.flip(pos);
            assert!(val.get(pos));
        }
    }
}
