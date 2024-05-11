use engine::{
    systems::{GameSystem, GameSystemCommand},
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityBundle, EntityID, Float,
    SizeFloat, Vec2f,
};

use crate::{
    pbm::PBMImage,
    resource::{
        PLAYER_SHOTGUN_IDLE_ANIM, WORLD_CANDELABRA, WORLD_LEVEL_BASIC, WORLD_TORCH_GREEN_ANIM,
        WORLD_TORCH_RED_ANIM,
    },
};

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
        frames: usize,
        storage: &mut ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> EngineResult<()> {
        storage.remove_all_entities();
        self.player_id = storage.append(&bundle_player(Vec2f::new(5.0, 10.0), frames));
        self.maze_id = storage.append(&bundle_maze(asset_manager)?);
        // decorations
        storage.append(&bundle_torch(
            TorchStyle::Green,
            Vec2f::new(1.2, 12.9),
            frames,
        ));
        storage.append(&bundle_torch(
            TorchStyle::Green,
            Vec2f::new(1.2, 4.1),
            frames,
        ));
        storage.append(&bundle_torch(TorchStyle::Red, Vec2f::new(1.2, 9.0), frames));
        storage.append(&bundle_sprite(WORLD_CANDELABRA, Vec2f::new(8.8, 2.8)));
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
        self.generate_level(0, storage, asset_manager)?;
        println!("[v2.generator] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        frames: usize,
        _delta_time: Float,
        storage: &mut ComponentStorage,
        asset_manager: &AssetManager,
    ) -> engine::EngineResult<GameSystemCommand> {
        // TODO: implement valid logic for (re)creating levels and characters
        if storage.has_component::<InvalidatedTag>(self.player_id) {
            self.generate_level(frames, storage, asset_manager)?;
        }

        Ok(GameSystemCommand::Nothing)
    }
}

fn bundle_player(position: Vec2f, frame: usize) -> EntityBundle {
    EntityBundle::new()
        .put(PlayerTag)
        .put(ControllerState::default())
        .put(Sprite::with_animation(
            PLAYER_SHOTGUN_IDLE_ANIM,
            frame,
            usize::MAX,
        ))
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

enum TorchStyle {
    Green,
    Red,
}

fn bundle_torch(style: TorchStyle, position: Vec2f, frame: usize) -> EntityBundle {
    let animation_id = match style {
        TorchStyle::Green => WORLD_TORCH_GREEN_ANIM,
        TorchStyle::Red => WORLD_TORCH_RED_ANIM,
    };
    EntityBundle::new()
        .put(Sprite::with_animation(animation_id, frame, usize::MAX))
        .put(Position(position))
        .put(ScaleRatio(0.7))
        .put(HeightShift(0.27))
        .put(BoundingBox(SizeFloat::new(0.3, 0.3)))
}

fn bundle_sprite(texture_id: &'static str, position: Vec2f) -> EntityBundle {
    EntityBundle::new()
        .put(Sprite::with_texture(texture_id))
        .put(Position(position))
        .put(ScaleRatio(0.7))
        .put(HeightShift(0.27))
}
