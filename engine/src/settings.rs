use crate::{assets::AssetSource, geometry::SizeU32};

pub struct EngineSettings {
    pub window: WindowSettings,
    pub asset_source: AssetSource,
}
pub struct WindowSettings {
    pub title: String,
    pub size: SizeU32,
}
