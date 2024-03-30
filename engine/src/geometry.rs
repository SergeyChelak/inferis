#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

pub type Float = f32;
pub type Vec2f = Vec2<Float>;

impl Vec2f {
    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }
}
