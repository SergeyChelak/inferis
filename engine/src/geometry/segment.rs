use crate::Float;

#[derive(Clone, Copy)]
pub struct Segment<T> {
    pub begin: T,
    pub length: T,
}

impl<T> Segment<T> {
    pub fn new(begin: T, length: T) -> Self {
        Self { begin, length }
    }
}

impl Segment<Float> {
    pub fn has_intersection(&self, other: &Self) -> bool {
        let (l, r) = if self.begin < other.begin {
            (self, other)
        } else {
            (other, self)
        };
        r.begin <= l.begin + l.length
    }
}
