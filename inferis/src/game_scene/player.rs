use engine::{
    frame_counter::AggregatedFrameCounter, game_scene::GameSystem, world::GameWorldState,
    ComponentStorage, EngineError, EngineResult, EntityID,
};

use crate::game_scene::fetch_player_id;

use super::components;

#[derive(Default)]
pub struct PlayerSystem {
    player_id: EntityID,
}

impl PlayerSystem {
    pub fn new() -> Self {
        Default::default()
    }

    fn handle_controls(
        &self,
        world_state: &mut GameWorldState,
        storage: &mut ComponentStorage,
    ) -> EngineResult<()> {
        let Some(comp) = storage.get::<components::ControllerState>(self.player_id) else {
            println!("[v2.controller] warn: controller component isn't associated with player");
            return Ok(());
        };
        if comp.exit_pressed {
            println!("world stop");
            world_state.stop();
        }
        Ok(())
    }
}

impl GameSystem for PlayerSystem {
    fn setup(
        &mut self,
        storage: &engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        let Some(player_id) = fetch_player_id(storage) else {
            return Err(EngineError::unexpected_state(
                "[v2.player] player entity not found",
            ));
        };
        self.player_id = player_id;
        println!("[v2.player] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        world_state: &mut GameWorldState,
        frame_counter: &mut AggregatedFrameCounter,
        delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        assets: &engine::AssetManager,
    ) -> engine::EngineResult<Vec<engine::game_scene::Effect>> {
        self.handle_controls(world_state, storage)?;
        Ok(vec![])
    }
}
