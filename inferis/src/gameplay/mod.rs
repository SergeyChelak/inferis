use engine::{ComponentStorage, EngineResult, Float, SizeFloat, Vec2f};

use crate::resource::*;

mod ai;
mod attack;
mod common;
mod controller;
mod frame_counter;
mod generator;
mod input;
pub mod main_scene;
mod renderer;
mod sound;
mod state;
mod transform;

pub use crate::game_scene::components::Angle;
pub use crate::game_scene::components::BoundingBox;
pub use crate::game_scene::components::Health;
pub use crate::game_scene::components::HealthType;
pub use crate::game_scene::components::HeightShift;
pub use crate::game_scene::components::InvalidatedTag;
pub use crate::game_scene::components::Maze;
pub use crate::game_scene::components::Movement;
pub use crate::game_scene::components::NpcTag;
pub use crate::game_scene::components::PlayerTag;
pub use crate::game_scene::components::Position;
pub use crate::game_scene::components::RotationSpeed;
pub use crate::game_scene::components::ScaleRatio;
pub use crate::game_scene::components::SoundFx;
pub use crate::game_scene::components::Velocity;
pub use crate::game_scene::components::Weapon;
pub use crate::game_scene::components::WeaponState;

pub struct UserControllableTag;
pub struct SpriteTag;

#[derive(Clone, Copy, Debug)]
pub enum CharacterState {
    Idle,
    Death,
    Attack,
    Walk,
    Damage,
}

#[derive(Clone, Copy)]
pub struct Shot {
    position: Vec2f,
    angle: Float,
    state: ShotState,
}

#[derive(Clone, Copy)]
pub enum ShotState {
    Initial,
    Processed,
}

pub struct ReceivedDamage(pub HealthType);

pub struct TextureID(pub String);

#[derive(Clone)]
pub struct AnimationData {
    pub frame_counter: usize,
    pub animation_id: String,
}

pub fn compose_component_storage() -> EngineResult<ComponentStorage> {
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
    storage.register_component::<Movement>()?;
    storage.register_component::<ReceivedDamage>()?;
    storage.register_component::<Shot>()?;
    storage.register_component::<Weapon>()?;
    storage.register_component::<UserControllableTag>()?;
    storage.register_component::<SoundFx>()?;
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

impl Shot {
    pub fn new(position: Vec2f, angle: Float) -> Self {
        Self {
            position,
            angle,
            state: ShotState::Initial,
        }
    }
}
