#[derive(Copy, Clone, Debug)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

pub type U32Size = Size<u32>;
