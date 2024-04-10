use crate::geometry::SizeU32;

pub struct EngineSettings {
    pub window: WindowSettings,
    pub asset_path: String,
}
pub struct WindowSettings {
    pub title: String,
    pub size: SizeU32,
}
