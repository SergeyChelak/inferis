use sdl2::{
    keyboard::Keycode,
    mixer::Chunk,
    render::{Texture, WindowCanvas},
};

use crate::{assets::AssetManager, EngineResult, SizeU32};

pub mod frame_counter;
pub mod game_loop;
pub mod game_scene;
pub mod systems;
pub mod world;

pub use frame_counter::FrameCounter;

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

// --- deprecated ---
pub trait Engine {
    fn change_scene(&mut self, scene_id: SceneID);
    fn terminate(&mut self);
    fn canvas(&mut self) -> &mut WindowCanvas;
    fn play_sound(&self, sound_chunk: &Chunk, loops: i32) -> EngineResult<()>;
    fn delta_time(&self) -> f32;
    fn window_size(&self) -> SizeU32;
}

pub trait Scene {
    fn process_events(&mut self, events: &[InputEvent]) -> EngineResult<()>;
    fn run_systems(&mut self, engine: &mut dyn Engine, assets: &AssetManager) -> EngineResult<()>;
    fn render_scene(&mut self, engine: &mut dyn Engine, assets: &AssetManager) -> EngineResult<()>;
    fn id(&self) -> SceneID;
}
