use std::{
    fs::{read_to_string, File},
    io::{self, Read},
};

use crate::{EngineError, EngineResult};

use super::{raw_asset::*, Data};

pub fn raw_assets_debug(path: &str) -> EngineResult<Vec<RawAsset>> {
    let (success, failures): (Vec<_>, Vec<_>) = read_to_string(path)
        .map_err(|e| {
            let msg = format!("Failed to read contents of file at {path}. Error: {}", e);
            EngineError::FileAccessError(msg)
        })?
        .lines()
        .filter(|x| !x.is_empty())
        .filter(|x| !x.starts_with('#'))
        .map(RawAsset::try_from)
        .partition(Result::is_ok);

    if !failures.is_empty() {
        let all_errors = failures
            .into_iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<String>>()
            .join("\n");
        let msg = format!("Failed to parse assets:\n{}", all_errors);
        return Err(EngineError::ResourceParseError(msg));
    }
    let assets = success
        .into_iter()
        .map(Result::unwrap)
        .collect::<Vec<RawAsset>>();
    Ok(assets)
}

const ASSET_KEY_TEXTURE: &str = "texture";
const ASSET_KEY_COLOR: &str = "color";
const ASSET_KEY_VERTICAL_GRADIENT: &str = "vertical_gradient";
const ASSET_KEY_ANIMATION: &str = "animation";
const ASSET_KEY_BINARY: &str = "binary";

impl TryFrom<&str> for RawAsset {
    type Error = EngineError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let tokens = value.split_whitespace().collect::<Vec<&str>>();
        let Some(&id) = tokens.first() else {
            return Err(EngineError::ResourceParseError("Empty input".to_string()));
        };
        match id {
            ASSET_KEY_TEXTURE => RawAsset::texture(&tokens),
            ASSET_KEY_ANIMATION => RawAsset::animation(&tokens),
            ASSET_KEY_BINARY => RawAsset::binary(&tokens),
            ASSET_KEY_COLOR => RawAsset::color(&tokens),
            ASSET_KEY_VERTICAL_GRADIENT => RawAsset::vertical_gradient(&tokens),
            _ => {
                let msg = format!("Unknown type {id}");
                Err(EngineError::ResourceParseError(msg))
            }
        }
    }
}

impl RawAsset {
    fn texture(tokens: &[&str]) -> EngineResult<Self> {
        Self::raw_binary(tokens, Type::Texture)
    }

    fn animation(tokens: &[&str]) -> EngineResult<Self> {
        Self::raw_text(tokens, Type::Animation)
    }

    fn binary(tokens: &[&str]) -> EngineResult<Self> {
        Self::raw_binary(tokens, Type::Binary)
    }

    fn color(tokens: &[&str]) -> EngineResult<Self> {
        Self::raw_text(tokens, Type::Color)
    }

    fn vertical_gradient(tokens: &[&str]) -> EngineResult<Self> {
        Self::raw_text(tokens, Type::VerticalGradient)
    }

    fn raw_binary(tokens: &[&str], asset_type: Type) -> EngineResult<Self> {
        let (Some(&id), Some(&path)) = (tokens.get(1), tokens.get(2)) else {
            return Err(EngineError::ResourceParseError(format!(
                "Two arguments are required for parser. Input: '{tokens:?}'"
            )));
        };
        let value = file_to_buffer(path).map_err(|e| {
            let msg = format!("Failed to load raw data at {path} with err {e}");
            EngineError::FileAccessError(msg)
        })?;
        Ok(Self {
            id: id.to_string(),
            representation: Representation::Binary { value },
            asset_type,
        })
    }

    fn raw_text(tokens: &[&str], asset_type: Type) -> EngineResult<Self> {
        let Some(id) = tokens.get(1) else {
            return Err(EngineError::ResourceParseError(format!(
                "Id not present. Input: '{tokens:?}'"
            )));
        };
        if tokens.len() < 3 {
            return Err(EngineError::ResourceParseError(format!(
                "Invalid input string: '{tokens:?}'"
            )));
        };
        let value = tokens[2..].join("\t");
        Ok(Self {
            id: id.to_string(),
            representation: Representation::Text { value },
            asset_type,
        })
    }
}

fn file_to_buffer(path: &str) -> io::Result<Data> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
