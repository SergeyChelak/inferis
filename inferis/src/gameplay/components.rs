use engine::geometry::{Float, Vec2f};

pub struct Health(pub u32);

pub struct PlayerTag;

pub struct Position(pub Vec2f);

pub struct NpcTag;

pub struct Velocity(pub Float);

pub struct RotationSpeed(pub Float);

pub struct LevelTiles {
    pub content: Vec<Vec<u32>>,
    pub size: usize,
}
