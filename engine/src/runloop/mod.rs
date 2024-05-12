use sdl2::{keyboard::Keycode, render::Texture};

use crate::SizeU32;

pub mod game_scene;
pub mod systems;
pub mod world;

pub use game_scene::*;
pub use systems::*;
pub use world::*;

pub type SceneID = &'static str;

pub enum InputEvent {
    Quit,
    Keyboard {
        code: Keycode,
        pressed: bool,
    },
    Mouse {
        x: i32,
        y: i32,
        x_rel: i32,
        y_rel: i32,
    },
}

pub fn texture_size(texture: &Texture) -> SizeU32 {
    let query = texture.query();
    SizeU32 {
        width: query.width,
        height: query.height,
    }
}
