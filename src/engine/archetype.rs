use std::mem::size_of;

type Representation = u64;

#[derive(Default)]
pub struct Archetype {
    data: Representation,
}

impl Archetype {
    pub fn with_position(position: usize) -> Self {
        let mut val = Self::default();
        val.enable(position);
        val
    }

    pub fn set(&mut self, position: usize, enabled: bool) {
        if enabled {
            self.enable(position)
        } else {
            self.disable(position)
        }
    }

    pub fn disable(&mut self, position: usize) {
        let val = 1 << position;
        self.data &= !val;
    }

    pub fn enable(&mut self, position: usize) {
        let val = 1 << position;
        self.data |= val;
    }

    pub fn get(&self, position: usize) -> bool {
        self.data & (1 << position) > 0
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

    pub fn all_enabled(&self) -> Vec<usize> {
        let size = Self::max_items();
        let mut result = Vec::with_capacity(size);
        for pos in 0..size {
            if (self.data >> pos) & 1 == 1 {
                result.push(pos);
            }
        }
        result
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

    #[test]
    fn archetype_all_enabled() {
        let mut arch = Archetype::default();
        let arr = [1_usize, 5, 10, 15];
        for val in arr {
            arch.enable(val);
        }
        let all_enabled = arch.all_enabled();
        assert_eq!(all_enabled, arr);
    }
}
