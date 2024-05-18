mod matrix;

use engine::{
    systems::{GameSystem, GameSystemCommand},
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityBundle, EntityID, Float,
    SizeFloat, Vec2f,
};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    pbm::PBMImage,
    resource::{WORLD_CANDELABRA, WORLD_LEVEL_BASIC, WORLD_TORCH_GREEN_ANIM, WORLD_TORCH_RED_ANIM},
};

use self::matrix::{
    fill_borders, generate_matrix, moore_neighborhood, noise_matrix, regions, MatrixElement,
};

use super::components::*;

pub const PLAYER_SHOTGUN_DAMAGE: HealthType = 27;
pub const PLAYER_SHOTGUN_RECHARGE_FRAMES: usize = 45;

pub const NPC_SOLDIER_SHOTGUN_DAMAGE: HealthType = 4;
pub const NPC_SOLDIER_SHOTGUN_RECHARGE_FRAMES: usize = 30;

const MATRIX_COLS: usize = 50;
const MATRIX_ROWS: usize = 50;
const TILE_WALL: MatrixElement = 1;
const TILE_FLOOR: MatrixElement = 0;
const REGION_THRESHOLD: usize = 3;

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

        let mut matrix = generate_matrix(
            MATRIX_ROWS,
            MATRIX_COLS,
            TILE_WALL,
            TILE_FLOOR,
            REGION_THRESHOLD,
        )
        .ok_or(EngineError::unexpected_state(
            "[v2.generator] failed to build new matrix",
        ))?;
        {
            // optional step: assign different wall textures
            let regions = matrix::regions(&matrix, TILE_WALL);
            for (idx, region) in regions.iter().enumerate() {
                for pos in region.iter() {
                    matrix[pos.row][pos.col] = 1 + (idx % 5) as i32;
                }
            }
        }
        let mut available_places = regions(&matrix, TILE_FLOOR)
            .first()
            .ok_or(EngineError::unexpected_state(
                "[v2.generator] no empty spaces",
            ))?
            .iter()
            .map(|p| Vec2f::new(p.col as Float, p.row as Float))
            .collect::<Vec<Vec2f>>();
        available_places.shuffle(&mut thread_rng());

        let maze_bundle = EntityBundle::new().put(Maze(matrix));
        self.maze_id = storage.append(&maze_bundle);

        let offset = Vec2f::new(0.5, 0.5);
        {
            let Some(pos) = available_places.pop() else {
                return Err(EngineError::unexpected_state(
                    "[v2.generator] no place for player position",
                ));
            };
            self.player_id = storage.append(&bundle_player(pos + offset));
        }
        /*
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
         */

        // npc
        let soldiers = 20;
        for _ in 0..soldiers {
            let Some(pos) = available_places.pop() else {
                break;
            };
            storage.append(&bundle_npc_soldier(pos + offset));
        }
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
        _frames: usize,
        _delta_time: Float,
        _storage: &mut ComponentStorage,
        _asset_manager: &AssetManager,
    ) -> engine::EngineResult<GameSystemCommand> {
        // TODO: implement valid logic for (re)creating levels and characters
        // if storage.has_component::<InvalidatedTag>(self.player_id) {
        //     self.generate_level(frames, storage, asset_manager)?;
        // }

        Ok(GameSystemCommand::Nothing)
    }
}

fn bundle_player(position: Vec2f) -> EntityBundle {
    EntityBundle::new()
        .put(PlayerTag)
        .put(ActorState::Undefined)
        .put(ControllerState::default())
        .put(weapon(
            PLAYER_SHOTGUN_DAMAGE,
            PLAYER_SHOTGUN_RECHARGE_FRAMES,
            usize::MAX,
        ))
        .put(Health(500))
        .put(Velocity(7.5))
        .put(RotationSpeed(2.5))
        .put(Position(position))
        .put(Angle(0.0))
        .put(BoundingBox(SizeFloat::new(0.7, 0.7)))
}

fn bundle_npc_soldier(position: Vec2f) -> EntityBundle {
    EntityBundle::new()
        .put(weapon(
            NPC_SOLDIER_SHOTGUN_DAMAGE,
            NPC_SOLDIER_SHOTGUN_RECHARGE_FRAMES,
            usize::MAX,
        ))
        .put(Position(position))
        .put(NpcTag)
        .put(ActorState::Undefined)
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
        state: WeaponState::Undefined,
        ammo_count,
    }
}

fn generate_maze() -> EngineResult<EntityBundle> {
    let noise_density = 58;
    let iterations = 3;
    let mut matrix = vec![vec![0; MATRIX_COLS]; MATRIX_ROWS];
    noise_matrix(&mut matrix, noise_density, TILE_WALL, TILE_FLOOR);
    fill_borders(&mut matrix, TILE_WALL);
    for _ in 0..iterations {
        let Some(new_matrix) = moore_neighborhood(&matrix, TILE_WALL, TILE_FLOOR) else {
            return Err(EngineError::unexpected_state(
                "[v2.generator] failed to build new matrix",
            ));
        };
        matrix = new_matrix;
    }
    // fill isolated gaps
    let mut regions = matrix::regions(&matrix, TILE_FLOOR);
    let max = regions.iter().map(|x| x.len()).max().unwrap_or_default();
    for region in regions.iter() {
        if region.len() == max {
            continue;
        }
        for pos in region.iter() {
            matrix[pos.row][pos.col] = TILE_WALL;
        }
    }
    // filter small wall regions
    regions = matrix::regions(&matrix, TILE_WALL);
    for region in regions.iter() {
        if region.len() > REGION_THRESHOLD {
            continue;
        }
        for pos in region.iter() {
            matrix[pos.row][pos.col] = TILE_FLOOR;
        }
    }
    // optional step: assign different wall textures
    regions = matrix::regions(&matrix, TILE_WALL);
    for (idx, region) in regions.iter().enumerate() {
        for pos in region.iter() {
            matrix[pos.row][pos.col] = 1 + (idx % 5) as i32;
        }
    }
    Ok(EntityBundle::new().put(Maze(matrix)))
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
