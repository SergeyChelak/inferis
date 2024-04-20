use engine::*;

use crate::resource::*;

use self::shot::perform_shots;

use super::{
    collider::run_collider, controller::ControllerState, level_generator::MazeGenerator,
    renderer::*, transform::transform_position, *,
};

pub struct GameScene {
    storage: ComponentStorage,
    controller: ControllerState,
    player_id: EntityID,
    maze_id: EntityID,
}

impl GameScene {
    pub fn new() -> EngineResult<Self> {
        let mut storage = game_play_component_storage()?;
        let player_id = {
            let position = Vec2f::new(5.0, 10.0);
            let bundle = EntityBundle::new()
                .put(PlayerTag)
                .put(Health(100))
                .put(Velocity(7.0))
                .put(RotationSpeed(2.5))
                .put(Position(position))
                .put(PrevPosition(position))
                .put(Angle(0.0))
                .put(TextureID(PLAYER_SHOTGUN.to_string()));
            storage.add_from_bundle(&bundle)
        };
        let maze_id = {
            let generator = MazeGenerator::default();
            let maze = generator.generate()?;
            let bundle = EntityBundle::new().put(maze);
            storage.add_from_bundle(&bundle)
        };
        {
            let bundle = EntityBundle::new()
                .put(AnimationData {
                    frame_counter: 0,
                    target_frames: usize::MAX,
                    animation_id: WORLD_TORCH_GREEN_ANIM.to_string(),
                })
                .put(Position(Vec2f::new(1.2, 12.9)))
                .put(ScaleRatio(0.7))
                .put(HeightShift(0.27))
                .put(SpriteTag);
            storage.add_from_bundle(&bundle);
        }
        {
            let bundle = EntityBundle::new()
                .put(AnimationData {
                    frame_counter: 0,
                    target_frames: usize::MAX,
                    animation_id: WORLD_TORCH_GREEN_ANIM.to_string(),
                })
                .put(Position(Vec2f::new(1.2, 4.1)))
                .put(ScaleRatio(0.7))
                .put(HeightShift(0.27))
                .put(SpriteTag);
            storage.add_from_bundle(&bundle);
        }
        {
            let bundle = EntityBundle::new()
                .put(AnimationData {
                    frame_counter: 30,
                    target_frames: usize::MAX,
                    animation_id: WORLD_TORCH_RED_ANIM.to_string(),
                })
                .put(Position(Vec2f::new(1.2, 9.0)))
                .put(ScaleRatio(0.7))
                .put(HeightShift(0.27))
                .put(SpriteTag);
            storage.add_from_bundle(&bundle);
        }
        Ok(Self {
            storage,
            controller: ControllerState::default(),
            player_id,
            maze_id,
        })
    }
}

impl Scene for GameScene {
    fn id(&self) -> String {
        "game_scene".to_string()
    }

    fn process_events(&mut self, events: &[InputEvent]) -> EngineResult<()> {
        self.controller.update(events);
        Ok(())
    }

    fn run_systems(&mut self, engine: &mut dyn Engine) -> EngineResult<()> {
        let delta_time = engine.delta_time();
        transform_position(
            &mut self.storage,
            self.player_id,
            &self.controller,
            delta_time,
        )?;
        perform_shots(&mut self.storage, self.player_id, &self.controller)?;
        // TODO: update NPC position
        run_collider(&mut self.storage, self.player_id, self.maze_id)?;
        Ok(())
    }

    fn render_scene(&mut self, engine: &mut dyn Engine, assets: &AssetManager) -> EngineResult<()> {
        let mut renderer = Renderer::new(
            &mut self.storage,
            engine,
            assets,
            self.player_id,
            self.maze_id,
        );
        renderer.render()
    }
}
