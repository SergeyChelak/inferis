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

    pub fn square_dist(&self) -> Float {
        self.x * self.x + self.y * self.y
    }

    pub fn hypotenuse(&self) -> Float {
        self.square_dist().sqrt()
    }
}

impl Display for Vec2f {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:5.3}; {:5.3}]", self.x, self.y)
    }
}
