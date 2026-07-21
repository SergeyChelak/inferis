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

    pub fn ray_intersect(&self, ray_pos: Vec2f, ray_dir: Vec2f) -> Option<Float> {
        let inv_dir_x = 1.0 / ray_dir.x;
        let inv_dir_y = 1.0 / ray_dir.y;

        let mut tmin = (self.position.x - ray_pos.x) * inv_dir_x;
        let mut tmax = (self.position.x + self.size.width - ray_pos.x) * inv_dir_x;

        if inv_dir_x < 0.0 {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (self.position.y - ray_pos.y) * inv_dir_y;
        let mut tymax = (self.position.y + self.size.height - ray_pos.y) * inv_dir_y;

        if inv_dir_y < 0.0 {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if tmin.is_nan() {
            tmin = 0.0;
        }
        if tmax.is_nan() {
            tmax = 0.0;
        }
        if tymin.is_nan() {
            tymin = 0.0;
        }
        if tymax.is_nan() {
            tymax = 0.0;
        }

        if (tmin > tymax) || (tymin > tmax) {
            return None;
        }

        if tymin > tmin {
            tmin = tymin;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        if tmax < 0.0 {
            return None;
        }

        let t = if tmin < 0.0 { tmax } else { tmin };
        Some(t)
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
