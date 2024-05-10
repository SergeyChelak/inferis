use engine::{
    systems::{GameSystem, GameSystemCommand},
    ComponentStorage, EngineError, EngineResult, EntityID,
};

use super::{
    components::{Angle, Health, RotationSpeed, Velocity},
    fetch_player_id,
};

#[derive(Default)]
pub struct GeneratorSystem {
    player_id: EntityID,
}

impl GeneratorSystem {
    pub fn new() -> Self {
        Self::default()
    }

    fn cache_player_id(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        self.player_id = fetch_player_id(storage).ok_or(EngineError::unexpected_state(
            "[v2.generator] player entity not found",
        ))?;
        Ok(())
    }

    fn generate_level(&mut self, storage: &mut ComponentStorage) -> EngineResult<()> {
        // setup player
        {
            storage.set(self.player_id, Some(Health(500)));
            storage.set(self.player_id, Some(Velocity(7.5)));
            storage.set(self.player_id, Some(RotationSpeed(2.5)));
            storage.set(self.player_id, Some(Angle(0.0)));
        }
        Ok(())
    }
}

impl GameSystem for GeneratorSystem {
    fn setup(
        &mut self,
        storage: &engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        self.cache_player_id(storage)?;
        println!("[v2.generator] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        frames: usize,
        delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        assets: &engine::AssetManager,
    ) -> engine::EngineResult<GameSystemCommand> {
        // TODO: implement valid logic for (re)creating levels and characters
        if !storage.has_component::<Health>(self.player_id) {
            self.generate_level(storage)?;
        }

        Ok(GameSystemCommand::Nothing)
    }
}
