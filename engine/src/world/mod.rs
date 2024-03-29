use sdl2::{keyboard::Keycode, render::WindowCanvas};

use crate::{assets::AssetManager, EngineResult, WindowSize};

pub mod game_world;

type SceneID = String;
pub trait Engine {
    fn change_scene(&mut self, scene_id: SceneID);
    fn terminate(&mut self);
    fn canvas(&mut self) -> &mut WindowCanvas;
    fn delta_time(&self) -> f32;
    fn window_size(&self) -> WindowSize;
}

pub trait Scene {
    fn teak(
        &mut self,
        engine: &mut dyn Engine,
        events: &[InputEvent],
        assets: &AssetManager,
    ) -> EngineResult<()>;
    fn id(&self) -> SceneID;
}

pub enum InputEvent {
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
