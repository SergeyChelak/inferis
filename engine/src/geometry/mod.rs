mod vec2f;
mod vector;

pub use vec2f::*;

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

pub type Float = f32;
