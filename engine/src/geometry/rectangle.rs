use std::fmt::Display;

use crate::{Float, SizeFloat, Vec2f};

use super::segment::Segment;

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub position: Vec2f,
    pub size: SizeFloat,
}

impl Rectangle {
    pub fn with_pole(pole: Vec2f, size: SizeFloat) -> Self {
        let x = pole.x - size.width * 0.5;
        let y = pole.y - size.height * 0.5;
        Self {
            position: Vec2f::new(x, y),
            size,
        }
    }

    pub fn contains(&self, point: Vec2f) -> bool {
        self.position.x <= point.x
            && self.position.x + self.size.width >= point.x
            && self.position.y <= point.y
            && self.position.y + self.size.height >= point.y
    }

    pub fn has_intersection(&self, other: &Rectangle) -> bool {
        let (seg_x, seg_y) = self.segments();
        let (other_seg_x, other_seg_y) = other.segments();
        seg_x.has_intersection(&other_seg_x) && seg_y.has_intersection(&other_seg_y)
    }

    fn segments(&self) -> (Segment<Float>, Segment<Float>) {
        (
            Segment::new(self.position.x, self.size.width),
            Segment::new(self.position.y, self.size.height),
        )
    }
}

impl Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let end = self.position + Vec2f::new(self.size.width, self.size.height);
        write!(f, "{} - {}", self.position, end)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rect_contains() {
        // [cast_shoot] ray (x:8, y:10) rect: [(7.5,9.5) - (8.5,10.5)]
        let rect = Rectangle {
            position: Vec2f::new(7.5, 9.5),
            size: SizeFloat::new(1.0, 1.0),
        };
        let point = Vec2f::new(8.0, 10.0);
        assert!(rect.contains(point))
    }
}
