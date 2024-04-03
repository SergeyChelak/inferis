// allowed types
// 1. textures
// 2. animations
// 3. sounds
// 4. fonts
// 5. colors

use std::{collections::HashMap, fs::read_to_string};

use sdl2::{
    image::LoadTexture,
    pixels::{Color, PixelFormatEnum},
    render::{Texture, TextureCreator},
    video::WindowContext,
};

use crate::{EngineError, EngineResult, Float};

pub struct AssetManager<'a> {
    textures: HashMap<String, Texture<'a>>,
    colors: HashMap<String, Color>,
}

impl<'a> AssetManager<'a> {
    pub fn new(
        filename: &str,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> EngineResult<Self> {
        let items = read_to_string(filename)
            .map_err(|e| {
                EngineError::FileAccessError(format!(
                    "Can't open file '{filename}' with error '{}'",
                    e.to_string()
                ))
            })?
            .lines()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        let mut textures: HashMap<String, Texture<'a>> = HashMap::default();
        let mut colors: HashMap<String, Color> = HashMap::default();
        for item in items {
            if item.starts_with('#') {
                continue;
            }
            let tokens = item.split_whitespace().collect::<Vec<&str>>();
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
                "texture" => {
                    let Ok(texture) = texture_creator.load_texture(value) else {
                        return Err(EngineError::ResourceParseError(format!(
                            "Failed to load texture at '{value}'"
                        )));
                    };
                    textures.insert(name.to_string(), texture);
                }
                "color" => {
                    let Ok(color) = parse_color(value) else {
                        return Err(EngineError::ResourceParseError(format!(
                            "Failed to parse color '{value}'"
                        )));
                    };
                    colors.insert(name.to_string(), color);
                }
                "vertical_gradient" => {
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
                _ => {
                    println!("[Assets] skipped '{tag}'")
                }
            }
        }
        Ok(Self { textures, colors })
    }

    pub fn texture(&self, key: &str) -> Option<&Texture> {
        self.textures.get(key)
    }

    pub fn color(&self, key: &str) -> Option<&Color> {
        self.colors.get(key)
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

fn create_gradient_texture<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    from: Color,
    to: Color,
    height: u32,
) -> EngineResult<Texture<'a>> {
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
