use std::path::Path;

use crate::common::read_file_as_lines;

use super::{EngineError, EngineResult};

// #[derive(Default)]
pub struct AssetManager {
    // allowed types
    // 1. textures
    // 2. animations
    // 3. sounds
    // 4. fonts
    // 5. colors
}

impl AssetManager {}

pub enum Asset {
    Texture {
        name: String,
        path: String,
    },
    Animation {
        name: String,
        path: String,
        frames: usize,
        duration: usize,
    },
    Font {
        name: String,
        path: String,
    },
    Sound {
        name: String,
        path: String,
    },
    Color {
        name: String,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
}

impl TryFrom<&str> for Asset {
    type Error = EngineError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parse_error = EngineError::AssetFileIncorrectRecord(value.to_string());
        let tokens = value.split_whitespace().collect::<Vec<&str>>();
        if tokens.len() < 3 {
            return Err(parse_error);
        }
        let name = tokens[1].to_string();
        let path = tokens[2].to_string();
        let type_name = tokens[0].to_lowercase();
        match type_name.as_ref() {
            "texture" => Ok(Asset::Texture { name, path }),
            "animation" => {
                if tokens.len() == 5 {
                    let err = parse_error.clone();
                    let frames = tokens[3].parse::<usize>().map_err(|_| err)?;
                    let duration = tokens[4].parse::<usize>().map_err(|_| parse_error)?;
                    Ok(Asset::Animation {
                        name,
                        path,
                        frames,
                        duration,
                    })
                } else {
                    Err(parse_error)
                }
            }
            "sound" => Ok(Asset::Sound { name, path }),
            "font" => Ok(Asset::Font { name, path }),
            "color" => {
                let components = path
                    .split(',')
                    .filter_map(|val| val.parse::<u8>().ok())
                    .collect::<Vec<u8>>();
                if components.len() == 4 {
                    Ok(Asset::Color {
                        name,
                        r: components[0],
                        g: components[1],
                        b: components[2],
                        a: components[3],
                    })
                } else {
                    Err(parse_error)
                }
            }
            _ => Err(parse_error),
        }
    }
}

impl Asset {
    pub fn read_configuration<P: AsRef<Path>>(path: P) -> EngineResult<Vec<Self>> {
        let lines = read_file_as_lines(path)
            .map_err(|err| crate::engine::EngineError::AssetFileReadFailed)?;
        let mut vec = Vec::new();
        for line in lines {
            // ignore comment lines
            if line.starts_with('#') {
                continue;
            }
            let asset = Asset::try_from(line.as_ref())?;
            vec.push(asset);
        }
        Ok(vec)
    }
}
