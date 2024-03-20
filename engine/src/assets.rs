// allowed types
// 1. textures
// 2. animations
// 3. sounds
// 4. fonts
// 5. colors

use std::collections::HashMap;

use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
    video::WindowContext,
};

pub struct AssetManager<'a> {
    textures: HashMap<String, Texture<'a>>,
}

impl<'a> AssetManager<'a> {
    pub fn new() -> Self {
        Self {
            textures: HashMap::default(),
        }
    }

    pub fn load(&mut self, texture_creator: &'a TextureCreator<WindowContext>) {
        let filename = "123";
        let key = "123";
        if let Ok(texture) = texture_creator.load_texture(filename) {
            self.textures.insert(key.to_string(), texture);
        }
    }

    pub fn texture(&self, key: &str) -> Option<&Texture> {
        self.textures.get(key)
    }
}
