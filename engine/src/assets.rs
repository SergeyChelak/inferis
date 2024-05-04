use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::Read,
};

use sdl2::{
    image::LoadTexture,
    pixels::{Color, PixelFormatEnum},
    render::{Texture, TextureCreator},
    video::WindowContext,
};

use crate::{EngineError, EngineResult, Float};

const ASSET_KEY_TEXTURE: &str = "texture";
const ASSET_KEY_COLOR: &str = "color";
const ASSET_KEY_VERTICAL_GRADIENT: &str = "vertical_gradient";
const ASSET_KEY_ANIMATION: &str = "animation";
const ASSET_KEY_BINARY: &str = "binary";

pub struct Animation {
    pub duration: u32, // duration in frames
    pub frames_count: usize,
    pub texture_id: String,
}

pub type Data = Vec<u8>;

pub struct AssetManager<'a> {
    textures: HashMap<String, Texture<'a>>,
    colors: HashMap<String, Color>,
    animations: HashMap<String, Animation>,
    binaries: HashMap<String, Data>,
}

impl<'a> AssetManager<'a> {
    pub fn new(
        filename: &str,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> EngineResult<Self> {
        let items = read_to_string(filename)
            .map_err(|e| {
                EngineError::FileAccessError(format!(
                    "Can't open file '{filename}' with error '{e}'"
                ))
            })?
            .lines()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let mut textures: HashMap<String, Texture<'a>> = HashMap::default();
        let mut colors: HashMap<String, Color> = HashMap::default();
        let mut animations: HashMap<String, Animation> = HashMap::default();
        let mut binaries: HashMap<String, Data> = HashMap::default();
        for item in items {
            if item.is_empty() || item.starts_with('#') {
                continue;
            }
            let tokens = item.split_whitespace().collect::<Vec<&str>>();
            #[allow(clippy::get_first)]
            let Some(&tag) = tokens.get(0) else {
                return Err(EngineError::ResourceParseError(format!(
                    "Tag not found in '{item}'"
                )));
            };
            let Some(&name) = tokens.get(1) else {
                return Err(EngineError::ResourceParseError(format!(
                    "Name not found in '{item}'"
                )));
            };
            let Some(&value) = tokens.get(2) else {
                return Err(EngineError::ResourceParseError(format!(
                    "Value not found in '{item}'"
                )));
            };
            match tag {
                ASSET_KEY_TEXTURE => {
                    let Ok(texture) = texture_creator.load_texture(value) else {
                        return Err(EngineError::ResourceParseError(format!(
                            "Failed to load texture at '{value}'"
                        )));
                    };
                    textures.insert(name.to_string(), texture);
                }
                ASSET_KEY_COLOR => {
                    let Ok(color) = parse_color(value) else {
                        return Err(EngineError::ResourceParseError(format!(
                            "Failed to parse color '{value}'"
                        )));
                    };
                    colors.insert(name.to_string(), color);
                }
                ASSET_KEY_VERTICAL_GRADIENT => {
                    let Some(height) = tokens.get(3).and_then(|&val| val.parse::<u32>().ok())
                    else {
                        return Err(EngineError::ResourceParseError(format!(
                            "Gradient height not found or invalid '{item}'"
                        )));
                    };
                    let (from, to) = parse_gradient(value)?;
                    let Ok(texture) = create_gradient_texture(texture_creator, from, to, height)
                    else {
                        return Err(EngineError::ResourceParseError(format!(
                            "Failed to create texture gradient '{value}'"
                        )));
                    };
                    textures.insert(name.to_string(), texture);
                }
                ASSET_KEY_ANIMATION => {
                    let Some(frames_count) =
                        tokens.get(3).and_then(|&val| val.parse::<usize>().ok())
                    else {
                        return Err(EngineError::ResourceParseError(format!(
                            "Failed to parse frames count in '{value}'"
                        )));
                    };
                    let Some(duration) = tokens.get(4).and_then(|&val| val.parse::<u32>().ok())
                    else {
                        return Err(EngineError::ResourceParseError(format!(
                            "Failed to parse animation duration in '{value}'"
                        )));
                    };
                    let animation = Animation {
                        frames_count,
                        duration,
                        texture_id: value.to_string(),
                    };
                    animations.insert(name.to_string(), animation);
                }
                ASSET_KEY_BINARY => {
                    let mut file = File::open(value)
                        .map_err(|e| EngineError::ResourceParseError(e.to_string()))?;
                    let mut buffer = Data::new();
                    file.read_to_end(&mut buffer)
                        .map_err(|e| EngineError::ResourceParseError(e.to_string()))?;
                    binaries.insert(name.to_string(), buffer);
                }
                _ => {
                    println!("[Assets] skipped '{tag}'")
                }
            }
        }
        Ok(Self {
            textures,
            colors,
            animations,
            binaries,
        })
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
