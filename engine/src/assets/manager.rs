use std::collections::HashMap;

use sdl2::{
    image::LoadTexture,
    mixer::*,
    pixels::{Color, PixelFormatEnum},
    render::{Texture, TextureCreator},
    rwops::{self, RWops},
    video::WindowContext,
};

use crate::{EngineError, EngineResult, Float};

use super::{
    bundle_parser::raw_assets_from_bundle,
    raw_asset::{RawAsset, Representation, Type},
    text_parser::raw_assets_from_text,
    AssetSource, AssetSourceType, Data,
};

pub struct Animation {
    pub duration: u32, // duration in frames
    pub frames_count: usize,
    pub texture_id: String,
}

#[derive(Default)]
pub struct AssetManager<'a> {
    textures: HashMap<String, Texture<'a>>,
    colors: HashMap<String, Color>,
    animations: HashMap<String, Animation>,
    binaries: HashMap<String, Data>,
    audio_chunks: HashMap<String, Chunk>,
}

impl<'a> AssetManager<'a> {
    pub fn setup(
        &mut self,
        source: &AssetSource,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> EngineResult<()> {
        let raw_assets = load_assets(source)?;
        for asset in &raw_assets {
            match asset.asset_type {
                Type::Texture => self.add_texture(asset, texture_creator)?,
                Type::Animation => self.add_animation(asset)?,
                Type::Binary => self.add_binary(asset)?,
                Type::Color => self.add_color(asset)?,
                Type::VerticalGradient => self.add_vertical_gradient(asset, texture_creator)?,
                Type::SoundChunk => self.add_audio_chunk(asset)?,
            }
        }
        Ok(())
    }

    fn add_texture(
        &mut self,
        raw_asset: &RawAsset,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> EngineResult<()> {
        let Representation::Binary { value } = &raw_asset.representation else {
            return Err(EngineError::UnexpectedState(format!(
                "Binary data not found for asset with id '{}'",
                raw_asset.id
            )));
        };
        let Ok(texture) = texture_creator.load_texture_bytes(value) else {
            return Err(EngineError::ResourceParseError(format!(
                "Failed to load texture for asset with id '{}'",
                raw_asset.id
            )));
        };
        self.textures.insert(raw_asset.id.clone(), texture);
        Ok(())
    }

    fn add_animation(&mut self, raw_asset: &RawAsset) -> EngineResult<()> {
        let Representation::Text { value } = &raw_asset.representation else {
            return Err(EngineError::UnexpectedState(format!(
                "Text data not found for asset with id '{}'",
                raw_asset.id
            )));
        };
        let tokens = value.split_whitespace().collect::<Vec<&str>>();
        #[allow(clippy::get_first)]
        let Some(texture_id) = tokens.get(0).map(|x| x.to_string()) else {
            return Err(EngineError::ResourceParseError(format!(
                "Failed to parse frames count in '{value}'"
            )));
        };

        let Some(frames_count) = tokens.get(1).and_then(|&val| val.parse::<usize>().ok()) else {
            return Err(EngineError::ResourceParseError(format!(
                "Failed to parse frames count in '{value}'"
            )));
        };
        let Some(duration) = tokens.get(2).and_then(|&val| val.parse::<u32>().ok()) else {
            return Err(EngineError::ResourceParseError(format!(
                "Failed to parse animation duration in '{value}'"
            )));
        };
        let animation = Animation {
            frames_count,
            duration,
            texture_id,
        };
        self.animations.insert(raw_asset.id.clone(), animation);
        Ok(())
    }

    fn add_binary(&mut self, raw_asset: &RawAsset) -> EngineResult<()> {
        let Representation::Binary { value } = &raw_asset.representation else {
            return Err(EngineError::UnexpectedState(format!(
                "Binary data not found for asset with id '{}'",
                raw_asset.id
            )));
        };
        self.binaries.insert(raw_asset.id.clone(), value.clone());
        Ok(())
    }

    fn add_color(&mut self, raw_asset: &RawAsset) -> EngineResult<()> {
        let Representation::Text { value } = &raw_asset.representation else {
            return Err(EngineError::UnexpectedState(format!(
                "Text data not found for asset with id '{}'",
                raw_asset.id
            )));
        };
        let Ok(color) = parse_color(value) else {
            return Err(EngineError::ResourceParseError(format!(
                "Failed to parse color '{value}'"
            )));
        };
        self.colors.insert(raw_asset.id.clone(), color);
        Ok(())
    }

    fn add_vertical_gradient(
        &mut self,
        raw_asset: &RawAsset,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> EngineResult<()> {
        let Representation::Text { value } = &raw_asset.representation else {
            return Err(EngineError::UnexpectedState(format!(
                "Text data not found for asset with id '{}'",
                raw_asset.id
            )));
        };
        let tokens = value.split_whitespace().collect::<Vec<&str>>();
        let Some(height) = tokens.get(1).and_then(|&val| val.parse::<u32>().ok()) else {
            return Err(EngineError::ResourceParseError(format!(
                "Gradient height not found or invalid '{}'",
                raw_asset.id
            )));
        };
        let (from, to) = parse_gradient(tokens.first().unwrap())?;
        let Ok(texture) = create_gradient_texture(texture_creator, from, to, height) else {
            return Err(EngineError::ResourceParseError(format!(
                "Failed to create texture gradient '{value}'"
            )));
        };
        self.textures.insert(raw_asset.id.clone(), texture);
        Ok(())
    }

    fn add_audio_chunk(&mut self, raw_asset: &RawAsset) -> EngineResult<()> {
        let Representation::Binary { value } = &raw_asset.representation else {
            return Err(EngineError::UnexpectedState(format!(
                "Audio chunk not found for asset with id '{}'",
                raw_asset.id
            )));
        };
        let chunk = create_sound_chunk(&value)?;
        self.audio_chunks.insert(raw_asset.id.clone(), chunk);
        Ok(())
    }

    pub fn texture(&self, key: &str) -> Option<&Texture> {
        self.textures.get(key)
    }

    pub fn color(&self, key: &str) -> Option<&Color> {
        self.colors.get(key)
    }

    pub fn animation(&self, key: &str) -> Option<&Animation> {
        self.animations.get(key)
    }

    pub fn binary(&self, key: &str) -> Option<&Data> {
        self.binaries.get(key)
    }

    pub fn sound_chunk(&self, key: &str) -> Option<&Chunk> {
        self.audio_chunks.get(key)
    }
}

fn load_assets(source: &AssetSource) -> EngineResult<Vec<RawAsset>> {
    match source.src_type {
        AssetSourceType::Folder => raw_assets_from_text(&source.value),
        AssetSourceType::Bundle => raw_assets_from_bundle(&source.value),
    }
}

fn parse_color(value: &str) -> EngineResult<Color> {
    let (comps, errors): (Vec<_>, Vec<_>) = value
        .split(',')
        .map(|comp| comp.parse::<u8>())
        .partition(Result::is_ok);
    if !errors.is_empty() || comps.len() < 3 {
        return Err(EngineError::ResourceParseError(format!(
            "Incorrect color string: '{value}'"
        )));
    }
    let comps = comps.into_iter().map(Result::unwrap).collect::<Vec<u8>>();
    let r = comps[0];
    let g = comps[1];
    let b = comps[2];
    let a = *comps.get(3).unwrap_or(&0xff);
    Ok(Color::RGBA(r, g, b, a))
}

fn parse_gradient(value: &str) -> EngineResult<(Color, Color)> {
    let Some((from, to)) = value.split_once('-').and_then(|(from, to)| {
        let Ok(f) = parse_color(from) else {
            return None;
        };
        let Ok(t) = parse_color(to) else {
            return None;
        };
        Some((f, t))
    }) else {
        return Err(EngineError::ResourceParseError(format!(
            "Failed to parse vertical gradient colors '{value}'"
        )));
    };
    Ok((from, to))
}

fn create_gradient_texture(
    texture_creator: &TextureCreator<WindowContext>,
    from: Color,
    to: Color,
    height: u32,
) -> EngineResult<Texture> {
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 1, height)
        .map_err(|e| EngineError::Sdl(e.to_string()))?;

    let height_float = height as Float;
    let step = |begin: u8, end: u8| -> (Float, bool) {
        ((begin.abs_diff(end)) as Float / height_float, begin < end)
    };

    let steps = [step(from.r, to.r), step(from.g, to.g), step(from.b, to.b)];
    let next_color = |initial: u8, step: u32, step_size: Float, is_natural: bool| -> u8 {
        let val = (step_size * step as Float) as u8;
        if is_natural {
            initial.saturating_add(val)
        } else {
            initial.saturating_sub(val)
        }
    };
    texture
        .with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for step in 0..height {
                let pos = step as usize * pitch;
                buffer[pos] = next_color(from.r, step, steps[0].0, steps[0].1);
                buffer[pos + 1] = next_color(from.g, step, steps[1].0, steps[1].1);
                buffer[pos + 2] = next_color(from.b, step, steps[2].0, steps[2].1);
            }
        })
        .map_err(|e| EngineError::Sdl(e.to_string()))?;
    Ok(texture)
}

fn create_sound_chunk(data: &[u8]) -> EngineResult<Chunk> {
    unsafe {
        let src = RWops::from_bytes(data).map_err(|e| {
            let msg = format!("Failed to create sound chunk source: {e}");
            EngineError::sdl(msg)
        })?;
        let raw = sdl2::sys::mixer::Mix_LoadWAV_RW(src.raw(), 0);
        if raw.is_null() {
            return Err(EngineError::sdl("Failed to load sound chunk"));
        }
        Ok(Chunk { raw, owned: true })
    }
}
