use sdl2::render::WindowCanvas;

use crate::assets::AssetManager;

pub mod game_world;

type SceneID = String;
pub trait Engine {
    fn change_scene(&mut self, scene_id: SceneID);
    fn terminate(&mut self);
    fn canvas(&mut self) -> &mut WindowCanvas;
}

pub trait Scene {
    fn teak(&mut self, engine: &mut dyn Engine, events: &[InputEvent], assets: &AssetManager);
    fn id(&self) -> SceneID;
}

pub enum InputEvent {
    Keyboard {
        code: i32,
        pressed: bool,
    },
    Mouse {
        x: i32,
        y: i32,
        x_rel: i32,
        y_rel: i32,
    },
}
