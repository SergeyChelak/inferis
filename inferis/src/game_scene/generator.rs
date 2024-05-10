use engine::{
    systems::{GameSystem, GameSystemCommand},
    ComponentStorage, EngineResult, EntityBundle, EntityID, Vec2f,
};

use super::components::*;

#[derive(Default)]
pub struct GeneratorSystem {
    player_id: EntityID,
}

impl GeneratorSystem {
    pub fn new() -> Self {
        Self::default()
    }

    fn generate_level(
        &mut self,
        storage: &mut ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> EngineResult<()> {
        storage.remove_all_entities();
        self.player_id = storage.append(&bundle_player(Vec2f::new(5.0, 10.0)));
        // TODO: ...
        Ok(())
    }
}

impl GameSystem for GeneratorSystem {
    fn setup(
        &mut self,
        storage: &mut engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        // self.cache_player_id(storage)?;
        self.generate_level(storage, asset_manager)?;
        println!("[v2.generator] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        frames: usize,
        delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<GameSystemCommand> {
        // TODO: implement valid logic for (re)creating levels and characters
        if storage.has_component::<InvalidatedTag>(self.player_id) {
            self.generate_level(storage, asset_manager)?;
        }

        Ok(GameSystemCommand::Nothing)
    }
}

fn bundle_player(position: Vec2f) -> EntityBundle {
    EntityBundle::new()
        .put(PlayerTag)
        .put(ControllerState::default())
        // .put(UserControllableTag)
        // .put(weapon(PLAYER_SHOTGUN_DAMAGE, 60, usize::MAX))
        .put(Health(500))
        .put(Velocity(7.5))
        .put(RotationSpeed(2.5))
        .put(Position(position))
        .put(Angle(0.0))
    // .put(BoundingBox(SizeFloat::new(0.7, 0.7)))
}
