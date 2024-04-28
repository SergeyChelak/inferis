mod ray_caster;
mod rectangle;
mod segment;
mod vec2f;
mod vector;

pub type Float = f32;

pub use ray_caster::*;
pub use rectangle::*;
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
