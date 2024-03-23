use engine::*;

use super::{
    components::*, controller::ControllerState, renderer::render_scene,
    transform::transform_position,
};

pub struct GameScene {
    storage: ComponentStorage,
    controller: ControllerState,
    player_id: EntityID,
}

impl GameScene {
    pub fn new() -> EngineResult<Self> {
        let mut storage = make_game_storage()?;
        let bundle = EntityBundle::new()
            .add(PlayerTag)
            .add(Health(100))
            .add(Velocity(50.0))
            .add(RotationSpeed(2.0))
            .add(Position(Vec2f::new(300.0, 150.0)))
            .add(Angle(0.0));
        let player_id = storage.add_from_bundle(&bundle);
        Ok(Self {
            storage,
            controller: ControllerState::default(),
            player_id,
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
        // TODO: find & resolve collisions
        render_scene(&mut self.storage, engine, assets, self.player_id)?;
        self.controller.reset_relative();
        Ok(())
    }

    fn id(&self) -> String {
        "game_scene".to_string()
    }
}

fn make_game_storage() -> EngineResult<ComponentStorage> {
    let mut storage = ComponentStorage::new();
    storage.register_component::<PlayerTag>()?;
    storage.register_component::<NpcTag>()?;
    storage.register_component::<Health>()?;
    storage.register_component::<Position>()?;
    storage.register_component::<Velocity>()?;
    storage.register_component::<RotationSpeed>()?;
    storage.register_component::<Maze>()?;
    storage.register_component::<Angle>()?;
    Ok(storage)
}
