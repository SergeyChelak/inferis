use engine::*;

use super::{
    collider::run_collider, controller::ControllerState, maze_generator::MazeGenerator,
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
                .put(Angle(0.0));
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
                .put(TextureID("candelabra".to_string()))
                .put(Position(Vec2f::new(6.0, 6.0)))
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
    fn teak(
        &mut self,
        engine: &mut dyn Engine,
        events: &[InputEvent],
        assets: &AssetManager,
    ) -> EngineResult<()> {
        self.controller.update(events);
        let delta_time = engine.delta_time();
        transform_position(
            &mut self.storage,
            self.player_id,
            &self.controller,
            delta_time,
        )?;
        // TODO: update NPC position
        run_collider(&mut self.storage, self.player_id, self.maze_id)?;
        render_scene(&self.storage, engine, assets, self.player_id, self.maze_id)?;
        self.controller.reset_relative();
        Ok(())
    }

    fn id(&self) -> String {
        "game_scene".to_string()
    }
}
