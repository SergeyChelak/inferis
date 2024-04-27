use engine::{ComponentStorage, EngineResult, Float, FrameCounter, SizeFloat, Vec2f};

use crate::resource::*;

mod attack;
mod controller;
mod input;
pub mod main_scene;
mod npc;
mod ray_caster;
mod renderer;
mod state;
mod transform;

pub type HealthType = u32;
pub struct Health(pub HealthType);

pub struct PlayerTag;

pub struct Position(pub Vec2f);

pub struct NpcTag;

#[derive(Clone, Copy, Debug)]
pub enum CharacterState {
    Idle(FrameCounter),
    Death(FrameCounter),
    Attack(FrameCounter),
    Walk(FrameCounter),
    Damage(FrameCounter),
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Transform {
    pub relative_x: Float,
    pub relative_y: Float,
    pub relative_angle: Float,
}

pub struct Velocity(pub Float);

pub struct RotationSpeed(pub Float);

pub struct Angle(pub Float);

#[derive(Clone, Copy)]

pub struct Shot {
    position: Vec2f,
    angle: Float,
    state: ShotState,
}

#[derive(Clone, Copy)]
pub enum ShotState {
    Initial,
    Accepted,
    Cancelled,
}

#[derive(Clone, Copy)]
pub struct Weapon {
    pub damage: HealthType,
    pub recharge_time: usize,
    pub ammo_count: usize,
    pub state: WeaponState,
}

#[derive(Clone, Copy)]
pub enum WeaponState {
    Ready,
    Recharge,
}

pub struct ReceivedDamage(pub HealthType);

pub type MazeData = Vec<Vec<i32>>;
pub struct Maze(pub MazeData);

pub struct TextureID(pub String);

pub struct SpriteTag;

pub struct ScaleRatio(pub Float);

pub struct HeightShift(pub Float);

#[derive(Clone)]
pub struct AnimationData {
    pub frame_counter: usize,
    pub animation_id: String,
}

pub struct BoundingBox(pub SizeFloat);

pub fn game_play_component_storage() -> EngineResult<ComponentStorage> {
    let mut storage = ComponentStorage::new();
    storage.register_component::<SpriteTag>()?;
    storage.register_component::<PlayerTag>()?;
    storage.register_component::<NpcTag>()?;
    storage.register_component::<CharacterState>()?;
    storage.register_component::<Health>()?;
    storage.register_component::<Position>()?;
    storage.register_component::<Velocity>()?;
    storage.register_component::<RotationSpeed>()?;
    storage.register_component::<Maze>()?;
    storage.register_component::<Angle>()?;
    storage.register_component::<TextureID>()?;
    storage.register_component::<ScaleRatio>()?;
    storage.register_component::<HeightShift>()?;
    storage.register_component::<AnimationData>()?;
    storage.register_component::<BoundingBox>()?;
    storage.register_component::<Transform>()?;
    storage.register_component::<ReceivedDamage>()?;
    storage.register_component::<Shot>()?;
    storage.register_component::<Weapon>()?;

    Ok(storage)
}

// the step back was done for AnimationData and Maze in philosophy of ECS
// there is no big difference if this implementation will be done with pure functions
// but just for convenience I decided to keep this code this way
impl AnimationData {
    pub fn new(animation_id: impl Into<String>) -> Self {
        Self {
            frame_counter: 0,
            animation_id: animation_id.into(),
        }
    }

    pub fn replace(&mut self, other_animation_id: &str) {
        if other_animation_id == self.animation_id {
            return;
        }
        self.animation_id = other_animation_id.to_string();
        self.frame_counter = 0;
    }
}

impl Maze {
    pub fn wall_texture(&self, point: Vec2f) -> Option<&str> {
        match self.value_at(point)? {
            1 => Some(WORLD_WALL1),
            2 => Some(WORLD_WALL2),
            3 => Some(WORLD_WALL3),
            4 => Some(WORLD_WALL4),
            5 => Some(WORLD_WALL5),
            _ => None,
        }
    }

    pub fn value_at(&self, point: Vec2f) -> Option<&i32> {
        let Vec2f { x, y } = point;
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let (col, row) = (point.x as usize, point.y as usize);
        self.0.get(row).and_then(|x| x.get(col))
    }

    pub fn is_wall(&self, point: Vec2f) -> bool {
        let Some(val) = self.value_at(point) else {
            return false;
        };
        *val != 0
    }
}
