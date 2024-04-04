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

    pub fn subtract(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn atan2(&self) -> Float {
        self.y.atan2(self.x)
    }

    pub fn square_dist(&self) -> Float {
        self.x * self.x + self.y * self.y
    }

    pub fn dist(&self) -> Float {
        self.square_dist().sqrt()
    }
}
