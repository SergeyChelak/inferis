use sdl2::rect::Rect;

use crate::{
    frame_counter::AggregatedFrameCounter, AssetManager, ComponentStorage, EngineResult, Float,
    InputEvent, SceneID, SizeU32,
};

pub enum GameSystemCommand {
    Nothing,
    SwitchScene(SceneID),
    Terminate,
}

pub trait GameSystem {
    fn setup(
        &mut self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<()>;
    fn update(
        &mut self,
        frame_counter: &mut AggregatedFrameCounter,
        delta_time: Float,
        storage: &mut ComponentStorage,
        assets: &AssetManager,
    ) -> EngineResult<GameSystemCommand>;
}

pub trait GameControlSystem {
    fn setup(&mut self, storage: &ComponentStorage) -> EngineResult<()>;
    fn push_events(
        &mut self,
        storage: &mut ComponentStorage,
        events: &[InputEvent],
    ) -> EngineResult<()>;
}

pub enum RendererEffect {
    RenderTexture {
        asset_id: String,
        source: Rect,
        destination: Rect,
    },
}

pub trait GameRendererSystem {
    fn setup(
        &mut self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
        window_size: SizeU32,
    ) -> EngineResult<()>;
    fn render(
        &mut self,
        frame_counter: &mut AggregatedFrameCounter,
        storage: &ComponentStorage,
        assets: &AssetManager,
    ) -> EngineResult<Vec<RendererEffect>>;
}

pub enum SoundEffect {
    PlaySound { asset_id: String, loops: i32 },
    // TODO: play music command
}

pub trait GameSoundSystem {
    fn setup(
        &mut self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<()>;

    fn update(
        &mut self,
        storage: &mut ComponentStorage,
        assets: &AssetManager,
    ) -> EngineResult<Vec<SoundEffect>>;
}
