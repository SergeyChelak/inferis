pub mod matrix;

use engine::{
    systems::{GameSystem, GameSystemCommand},
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityBundle, EntityID, Float,
    SizeFloat, Vec2f, RAY_CASTER_MAX_DEPTH,
};
use rand::{seq::SliceRandom, thread_rng};

use crate::resource::{WORLD_TORCH_GREEN_ANIM, WORLD_TORCH_RED_ANIM};

use self::matrix::{contours, generate_matrix, regions, MatrixElement};

use super::components::*;

pub const PLAYER_SHOTGUN_DAMAGE: HealthType = 27;
pub const PLAYER_SHOTGUN_RECHARGE_FRAMES: usize = 45;

pub const NPC_SOLDIER_SHOTGUN_DAMAGE: HealthType = 4;
pub const NPC_SOLDIER_SHOTGUN_RECHARGE_FRAMES: usize = 30;

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
    ) -> EngineResult<()> {
        storage.remove_all_entities();
        let mut matrix = generate_matrix(
            RAY_CASTER_MAX_DEPTH,
            RAY_CASTER_MAX_DEPTH,
            TILE_WALL,
            TILE_FLOOR,
            REGION_THRESHOLD,
        )
        .ok_or(EngineError::unexpected_state(
            "[v2.generator] failed to build new matrix",
        ))?;
        let contour = contours(&matrix, TILE_WALL);
        {
            // optional step: assign different wall textures
            let regions = matrix::regions(&matrix, TILE_WALL);
            for (idx, region) in regions.iter().enumerate() {
                for pos in region.iter() {
                    matrix[pos.row][pos.col] = 1 + (idx % WALL_TEXTURES.len()) as i32;
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

        let offset = Vec2f::new(0.5, 0.5);
        {
            let Some(pos) = available_places.pop() else {
                return Err(EngineError::unexpected_state(
                    "[v2.generator] no place for player position",
                ));
            };
            self.player_id = storage.append(&bundle_player(pos + offset));
        }
        // npc
        #[cfg(not(debug_assertions))]
        let soldiers = 20;
        #[cfg(debug_assertions)]
        let soldiers = 5;
        for _ in 0..soldiers {
            let Some(pos) = available_places.pop() else {
                break;
            };
            storage.append(&bundle_npc_soldier(pos + offset));
        }

        let maze = Maze { matrix, contour };

        // decorations
        #[cfg(not(debug_assertions))]
        let mut decorations = 30;
        #[cfg(debug_assertions)]
        let mut decorations = 15;
        let dy = Vec2f::new(0.0, 1.0);
        let dx = Vec2f::new(1.0, 0.0);
        for pos in available_places.iter() {
            let top = maze.is_wall(*pos - dy);
            let left = maze.is_wall(*pos - dx);
            let bottom = maze.is_wall(*pos + dy);
            let right = maze.is_wall(*pos + dx);
            if left && top {
                storage.append(&bundle_torch(
                    TorchStyle::Green,
                    *pos + Vec2f::new(0.1, 0.1),
                    frames,
                ));
            } else if top && right {
                storage.append(&bundle_torch(
                    TorchStyle::Red,
                    *pos + Vec2f::new(0.9, 0.1),
                    frames,
                ));
            } else if bottom && left {
                storage.append(&bundle_torch(
                    TorchStyle::Red,
                    *pos + Vec2f::new(0.1, 0.9),
                    frames,
                ));
            } else if bottom && right {
                storage.append(&bundle_torch(
                    TorchStyle::Green,
                    *pos + Vec2f::new(0.9, 0.9),
                    frames,
                ));
            } else {
                continue;
            }
            decorations -= 1;
            if decorations == 0 {
                break;
            }
        }

        let maze_bundle = EntityBundle::new().put(maze);
        self.maze_id = storage.append(&maze_bundle);
        Ok(())
    }
}

impl GameSystem for GeneratorSystem {
    fn setup(
        &mut self,
        storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        self.generate_level(0, storage)?;
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

    fn on_scene_event(
        &mut self,
        _event: engine::game_scene::SceneEvent,
        _params: &engine::game_scene::SceneParameters,
    ) {
        todo!("scene change wasn't handled")
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
    // .put(BoundingBox(SizeFloat::new(0.3, 0.3)))
}
