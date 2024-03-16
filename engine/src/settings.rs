use crate::geometry::Size;

pub type WindowSize = Size<u32>;

pub struct WindowSettings {
    pub title: String,
    pub size: WindowSize,
}
