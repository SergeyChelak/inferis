// allowed types
// 1. textures
// 2. animations
// 3. sounds
// 4. fonts
// 5. colors

use std::{collections::HashMap, fs::read_to_string};

use sdl2::{
    image::LoadTexture,
    pixels::Color,
    render::{Texture, TextureCreator},
    video::WindowContext,
};

use crate::{EngineError, EngineResult};

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
