use std::fmt::Display;

use crate::Float;

use super::vector::Vec2;

pub type Vec2f = Vec2<Float>;

impl Vec2f {
    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    /// Squared magnitude of the vector. Prefer it over [`Self::length`] when
    /// only comparing magnitudes: it skips the square root.
    pub fn length_squared(&self) -> Float {
        self.x * self.x + self.y * self.y
    }

    /// Magnitude of the vector.
    pub fn length(&self) -> Float {
        self.length_squared().sqrt()
    }
}

impl Display for Vec2f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:5.3}; {:5.3}]", self.x, self.y)
    }
}
