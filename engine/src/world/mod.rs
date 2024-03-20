use sdl2::render::WindowCanvas;

use crate::assets::AssetManager;

pub mod game_world;
pub mod scene;

type SceneID = String;
pub trait Engine {
    fn change_scene(&mut self, scene_id: SceneID);
    fn terminate(&mut self);
    fn canvas(&mut self) -> &mut WindowCanvas;
}

pub trait Scene {
    fn update(&mut self, engine: &mut dyn Engine);
    fn render(&self, engine: &mut dyn Engine, assets: &AssetManager);
}
