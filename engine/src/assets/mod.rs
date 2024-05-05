mod bundle_parser;
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
    use crate::EngineError;

    use super::Data;

    pub type TypeID = u8;

    pub const REPRESENTATION_BINARY: TypeID = 0;
    pub const REPRESENTATION_TEXT: TypeID = 1;

    pub const TYPE_ID_TEXTURE: TypeID = 0;
    pub const TYPE_ID_ANIMATION: TypeID = 1;
    pub const TYPE_ID_BINARY: TypeID = 2;
    pub const TYPE_ID_COLOR: TypeID = 3;
    pub const TYPE_ID_VERTICAL_GRADIENT: TypeID = 4;

    #[derive(Debug)]
    pub enum Representation {
        Text { value: String },
        Binary { value: Data },
    }

    impl Representation {
        pub fn id(&self) -> TypeID {
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

    impl From<Type> for TypeID {
        fn from(value: Type) -> Self {
            use Type::*;
            match value {
                Texture => TYPE_ID_TEXTURE,
                Animation => TYPE_ID_ANIMATION,
                Binary => TYPE_ID_BINARY,
                Color => TYPE_ID_COLOR,
                VerticalGradient => TYPE_ID_VERTICAL_GRADIENT,
            }
        }
    }

    impl TryFrom<TypeID> for Type {
        type Error = EngineError;

        fn try_from(value: TypeID) -> Result<Self, Self::Error> {
            match value {
                TYPE_ID_TEXTURE => Ok(Self::Texture),
                TYPE_ID_ANIMATION => Ok(Self::Animation),
                TYPE_ID_BINARY => Ok(Self::Binary),
                TYPE_ID_COLOR => Ok(Self::Color),
                TYPE_ID_VERTICAL_GRADIENT => Ok(Self::VerticalGradient),
                _ => {
                    let msg = format!("unexpected asset type {}", value);
                    Err(EngineError::ResourceParseError(msg))
                }
            }
        }
    }

    #[derive(Debug)]
    pub struct RawAsset {
        pub asset_type: Type,
        pub id: String,
        pub representation: Representation,
    }
}
