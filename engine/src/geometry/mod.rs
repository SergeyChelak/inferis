pub mod ray_caster;
mod vec2f;
mod vector;

pub type Float = f32;

pub use vec2f::*;

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

pub type SizeU32 = Size<u32>;
pub type SizeFloat = Size<Float>;

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
