mod manager;
mod text_parser;
pub use manager::*;

pub type Data = Vec<u8>;

pub enum AssetSourceType {
    Bundle,
    Folder,
}

pub struct AssetSource {
    src_type: AssetSourceType,
    value: String,
}

impl AssetSource {
    pub fn with_bundle(path: impl Into<String>) -> Self {
        Self {
            src_type: AssetSourceType::Bundle,
            value: path.into(),
        }
    }

    pub fn with_folder(path: impl Into<String>) -> Self {
        Self {
            src_type: AssetSourceType::Folder,
            value: path.into(),
        }
    }
}

mod raw_asset {
    use super::Data;

    #[derive(Debug)]
    pub enum Representation {
        Text { value: String },
        Binary { value: Data },
    }

    #[derive(Debug)]
    pub enum Type {
        Texture,
        Animation,
        Binary,
        Color,
        VerticalGradient,
    }

    #[derive(Debug)]
    pub struct RawAsset {
        pub asset_type: Type,
        pub id: String,
        pub representation: Representation,
    }
}
