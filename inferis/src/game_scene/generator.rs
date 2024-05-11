use engine::{
    systems::{GameSystem, GameSystemCommand},
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityBundle, EntityID, Float,
    SizeFloat, Vec2f,
};

use crate::{pbm::PBMImage, resource::WORLD_LEVEL_BASIC};

use super::components::*;

#[derive(Default)]
pub struct GeneratorSystem {
    player_id: EntityID,
    maze_id: EntityID,
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
        self.maze_id = storage.append(&bundle_maze(asset_manager)?);
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
        self.generate_level(storage, asset_manager)?;
        println!("[v2.generator] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        frames: usize,
        delta_time: Float,
        storage: &mut ComponentStorage,
        asset_manager: &AssetManager,
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
        .put(BoundingBox(SizeFloat::new(0.7, 0.7)))
}

fn bundle_maze(asset_manager: &AssetManager) -> EngineResult<EntityBundle> {
    let Some(data) = asset_manager.binary(WORLD_LEVEL_BASIC) else {
        return Err(EngineError::MazeGenerationFailed(
            "Level map not found".to_string(),
        ));
    };
    let image = PBMImage::with_binary(data.clone())
        .map_err(|err| EngineError::MazeGenerationFailed(err.to_string()))?;
    let array = image.transform_to_array(|x| x as i32);
    Ok(EntityBundle::new().put(Maze(array)))
}
