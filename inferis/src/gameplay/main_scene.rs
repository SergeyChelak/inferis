use engine::{frame_counter::FrameCounterService, *};

use crate::resource::*;

use self::{
    ai::ai_system,
    attack::attack_system,
    generator::{generator_system, LevelData},
    state::{cleanup_system, state_system},
    transform::transform_entities,
};

use super::{controller::ControllerState, input::*, renderer::*, *};

pub struct GameScene {
    storage: ComponentStorage,
    controller: ControllerState,
    frame_counter: FrameCounterService,
    context: Option<LevelData>,
}

impl GameScene {
    pub fn new() -> EngineResult<Self> {
        Ok(Self {
            storage: game_play_component_storage()?,
            controller: ControllerState::default(),
            frame_counter: FrameCounterService::default(),
            context: Default::default(),
        })
    }
}

impl Scene for GameScene {
    fn id(&self) -> SceneID {
        SCENE_GAME_PLAY
    }

    fn process_events(&mut self, events: &[InputEvent]) -> EngineResult<()> {
        self.controller.update(events);
        Ok(())
    }

    fn run_systems(&mut self, engine: &mut dyn Engine, assets: &AssetManager) -> EngineResult<()> {
        cleanup_system(&mut self.storage)?;
        let level_data = generator_system(&mut self.storage, assets)?;
        let delta_time = engine.delta_time();
        user_input_system(
            &mut self.storage,
            &self.controller,
            delta_time,
            level_data.player_id,
        )?;
        ai_system(&mut self.storage, level_data.player_id, delta_time)?;
        transform_entities(&mut self.storage)?;
        attack_system(&mut self.storage, &mut self.frame_counter)?;
        state_system(&mut self.storage, &mut self.frame_counter)?;
        self.frame_counter.teak();
        self.context = Some(level_data);
        Ok(())
    }

    fn render_scene(&mut self, engine: &mut dyn Engine, assets: &AssetManager) -> EngineResult<()> {
        let Some(level_data) = &self.context else {
            panic!()
        };
        let mut renderer = Renderer::new(
            &mut self.storage,
            engine,
            assets,
            level_data.player_id,
            level_data.maze_id,
        );
        renderer.render()
    }
}
