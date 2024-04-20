use engine::{ComponentStorage, EngineResult, Float, SizeFloat, Vec2f};

mod collider;
mod controller;
pub mod main_scene;
mod npc;
mod ray_caster;
mod renderer;
mod shot;
mod transform;

pub struct Health(pub u32);

pub struct PlayerTag;

pub struct Position(pub Vec2f);

pub struct PrevPosition(pub Vec2f);

pub struct NpcTag;

pub struct NpcDisplayMode(pub npc::State);

pub struct Velocity(pub Float);

pub struct RotationSpeed(pub Float);

pub struct Angle(pub Float);

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
    pub target_frames: usize,
}

impl AnimationData {
    pub fn new(animation_id: impl Into<String>, target_frames: usize) -> Self {
        Self {
            frame_counter: 0,
            animation_id: animation_id.into(),
            target_frames,
        }
    }

    pub fn endless(animation_id: impl Into<String>) -> Self {
        Self::new(animation_id, usize::MAX)
    }
}

pub struct BoundingBox(pub SizeFloat);

pub fn game_play_component_storage() -> EngineResult<ComponentStorage> {
    let mut storage = ComponentStorage::new();
    storage.register_component::<SpriteTag>()?;
    storage.register_component::<PlayerTag>()?;
    storage.register_component::<NpcTag>()?;
    storage.register_component::<NpcDisplayMode>()?;
    storage.register_component::<Health>()?;
    storage.register_component::<Position>()?;
    storage.register_component::<Velocity>()?;
    storage.register_component::<RotationSpeed>()?;
    storage.register_component::<Maze>()?;
    storage.register_component::<Angle>()?;
    storage.register_component::<PrevPosition>()?;
    storage.register_component::<TextureID>()?;
    storage.register_component::<ScaleRatio>()?;
    storage.register_component::<HeightShift>()?;
    storage.register_component::<AnimationData>()?;
    storage.register_component::<BoundingBox>()?;
    Ok(storage)
}
