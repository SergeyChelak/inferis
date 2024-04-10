use engine::*;

use super::{
    collider::run_collider, components::*, controller::ControllerState,
    maze_generator::MazeGenerator, renderer::*, transform::transform_position,
};

pub struct GameScene {
    storage: ComponentStorage,
    controller: ControllerState,
    player_id: EntityID,
    maze_id: EntityID,
}

impl GameScene {
    pub fn new() -> EngineResult<Self> {
        let mut storage = {
            let mut storage = ComponentStorage::new();
            storage.register_component::<SpriteTag>()?;
            storage.register_component::<PlayerTag>()?;
            storage.register_component::<NpcTag>()?;
            storage.register_component::<Health>()?;
            storage.register_component::<Position>()?;
            storage.register_component::<Velocity>()?;
            storage.register_component::<RotationSpeed>()?;
            storage.register_component::<Maze>()?;
            storage.register_component::<Angle>()?;
            storage.register_component::<PrevPosition>()?;
            storage.register_component::<TextureID>()?;
            storage.register_component::<ScaleRatio>()?;
            Ok(storage)
        }?;
        let player_id = {
            let position = Vec2f::new(5.0, 10.0);
            let bundle = EntityBundle::new()
                .add(PlayerTag)
                .add(Health(100))
                .add(Velocity(7.0))
                .add(RotationSpeed(2.5))
                .add(Position(position))
                .add(PrevPosition(position))
                .add(Angle(0.0));
            storage.add_from_bundle(&bundle)
        };
        let maze_id = {
            let generator = MazeGenerator::default();
            let maze = generator.generate()?;
            let bundle = EntityBundle::new().add(maze);
            storage.add_from_bundle(&bundle)
        };
        {
            let bundle = EntityBundle::new()
                .add(TextureID("candelabra".to_string()))
                .add(Position(Vec2f::new(6.0, 6.0)))
                .add(ScaleRatio(0.7))
                .add(SpriteTag);
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
        render_scene(
            &mut self.storage,
            engine,
            assets,
            self.player_id,
            self.maze_id,
        )?;
        self.controller.reset_relative();
        Ok(())
    }

    fn id(&self) -> String {
        "game_scene".to_string()
    }
}
