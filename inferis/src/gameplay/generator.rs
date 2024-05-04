use std::any::Any;

use engine::{
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityBundle, EntityID, Query, Vec2f,
};

use crate::{gameplay::Maze, pbm::PBMImage};

use super::*;

pub struct LevelData {
    pub player_id: EntityID,
    pub maze_id: EntityID,
}

pub fn generator_system(
    storage: &mut ComponentStorage,
    assets: &AssetManager,
) -> EngineResult<LevelData> {
    if let Some(level_data) = valid_level_data(storage) {
        return Ok(level_data);
    };
    generate_level(storage, assets)
}

fn generate_level(
    storage: &mut ComponentStorage,
    assets: &AssetManager,
) -> EngineResult<LevelData> {
    storage.remove_all_entities();
    let player_id = storage.append(&bundle_player(Vec2f::new(5.0, 10.0)));
    let maze_id = storage.append(&bundle_maze(assets)?);
    // decorations
    storage.append(&bundle_torch(TorchStyle::Green, Vec2f::new(1.2, 12.9)));
    storage.append(&bundle_torch(TorchStyle::Green, Vec2f::new(1.2, 4.1)));
    storage.append(&bundle_torch(TorchStyle::Red, Vec2f::new(1.2, 9.0)));
    storage.append(&bundle_sprite(WORLD_CANDELABRA, Vec2f::new(8.8, 2.8)));

    // npc
    [
        Vec2f::new(27.0, 13.8),
        Vec2f::new(8.0, 10.0),
        Vec2f::new(40.0, 8.0),
        Vec2f::new(32.0, 23.0),
        Vec2f::new(40.0, 22.5),
        Vec2f::new(3.0, 12.5),
        Vec2f::new(11.5, 2.5),
        Vec2f::new(19.5, 1.5),
        Vec2f::new(40.5, 4.5),
    ]
    .iter()
    .for_each(|&v| {
        storage.append(&bundle_npc_soldier(v));
    });

    Ok(LevelData { player_id, maze_id })
}

fn valid_level_data(storage: &mut ComponentStorage) -> Option<LevelData> {
    let Some(player_id) = valid_id::<PlayerTag>(storage) else {
        return None;
    };
    let Some(maze_id) = valid_id::<Maze>(storage) else {
        return None;
    };
    Some(LevelData { player_id, maze_id })
}

fn valid_id<T: Any>(storage: &mut ComponentStorage) -> Option<EntityID> {
    let query = Query::new().with_component::<T>();
    let Some(&entity_id) = storage.fetch_entities(&query).first() else {
        return None;
    };
    if storage.has_component::<InvalidatedTag>(entity_id) {
        return None;
    }
    Some(entity_id)
}

// temporary producer functions
fn bundle_player(position: Vec2f) -> EntityBundle {
    EntityBundle::new()
        .put(PlayerTag)
        .put(UserControllableTag)
        .put(weapon(PLAYER_SHOTGUN_DAMAGE, 60, usize::MAX))
        .put(Health(500))
        .put(Velocity(7.5))
        .put(RotationSpeed(2.5))
        .put(Position(position))
        .put(Angle(0.0))
        .put(BoundingBox(SizeFloat::new(0.7, 0.7)))
}

fn bundle_maze(assets: &AssetManager) -> EngineResult<EntityBundle> {
    let Some(data) = assets.binary(WORLD_LEVEL_BASIC) else {
        return Err(EngineError::MazeGenerationFailed(
            "Level map not found".to_string(),
        ));
    };
    let image = PBMImage::with_binary(data.clone())
        .map_err(|err| EngineError::MazeGenerationFailed(err.to_string()))?;
    let array = image.transform_to_array(|x| x as i32);
    Ok(EntityBundle::new().put(Maze(array)))
}

fn bundle_npc_soldier(position: Vec2f) -> EntityBundle {
    EntityBundle::new()
        .put(SpriteTag)
        .put(weapon(4, 30, usize::MAX))
        .put(Position(position))
        .put(NpcTag)
        .put(CharacterState::Idle)
        .put(Health(100))
        .put(ScaleRatio(0.7))
        .put(HeightShift(0.27))
        .put(BoundingBox(SizeFloat::new(0.7, 0.7)))
        .put(Velocity(4.3))
}

fn weapon(damage: HealthType, recharge_time: usize, ammo_count: usize) -> Weapon {
    Weapon {
        damage,
        recharge_time,
        state: WeaponState::Ready,
        ammo_count,
    }
}

enum TorchStyle {
    Green,
    Red,
}

fn bundle_torch(style: TorchStyle, position: Vec2f) -> EntityBundle {
    let animation_id = match style {
        TorchStyle::Green => WORLD_TORCH_GREEN_ANIM,
        TorchStyle::Red => WORLD_TORCH_RED_ANIM,
    }
    .to_string();
    EntityBundle::new()
        .put(AnimationData::new(animation_id))
        .put(Position(position))
        .put(ScaleRatio(0.7))
        .put(HeightShift(0.27))
        .put(BoundingBox(SizeFloat::new(0.3, 0.3)))
        .put(SpriteTag)
}

fn bundle_sprite(texture_id: &str, position: Vec2f) -> EntityBundle {
    EntityBundle::new()
        .put(TextureID(texture_id.to_string()))
        .put(Position(position))
        .put(ScaleRatio(0.7))
        .put(HeightShift(0.27))
        .put(SpriteTag)
}
