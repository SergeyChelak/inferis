mod manager;
mod text_parser;
pub use manager::*;
pub use text_parser::raw_assets_from_text;

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

pub mod raw_asset {
    use super::Data;

    pub type AssetTypeID = u8;

    pub const REPRESENTATION_BINARY: AssetTypeID = 0;
    pub const REPRESENTATION_TEXT: AssetTypeID = 1;

    #[derive(Debug)]
    pub enum Representation {
        Text { value: String },
        Binary { value: Data },
    }

    impl Representation {
        pub fn id(&self) -> AssetTypeID {
            match self {
                Representation::Binary { .. } => REPRESENTATION_BINARY,
                Representation::Text { .. } => REPRESENTATION_TEXT,
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Type {
        Texture,
        Animation,
        Binary,
        Color,
        VerticalGradient,
    }

    impl Type {
        pub fn id(&self) -> AssetTypeID {
            *self as AssetTypeID
        }
    }

    #[derive(Debug)]
    pub struct RawAsset {
        pub asset_type: Type,
        pub id: String,
        pub representation: Representation,
    }
}
