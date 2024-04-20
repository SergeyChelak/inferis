use engine::{ComponentStorage, EngineResult, Float, Vec2f};

mod collider;
mod controller;
pub mod main_scene;
mod maze_generator;
mod ray_caster;
mod renderer;
mod shot;
mod transform;

mod resource {
    use engine::Vec2f;

    use super::MazeData;

    // world
    const WALL1: &str = &"wall1";
    const WALL2: &str = &"wall2";
    const WALL3: &str = &"wall3";
    const WALL4: &str = &"wall4";
    const WALL5: &str = &"wall5";

    pub const SKY: &str = "sky";

    pub fn wall_texture(point: Vec2f, maze: &MazeData) -> Option<&str> {
        let Vec2f { x, y } = point;
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let (col, row) = (point.x as usize, point.y as usize);
        let value = maze.get(row).and_then(|x| x.get(col))?;
        match value {
            1 => Some(WALL1),
            2 => Some(WALL2),
            3 => Some(WALL3),
            4 => Some(WALL4),
            5 => Some(WALL5),
            _ => None,
        }
    }
}

pub struct Health(pub u32);

pub struct PlayerTag;

pub struct Position(pub Vec2f);

pub struct PrevPosition(pub Vec2f);

pub struct NpcTag;

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

pub fn game_play_component_storage() -> EngineResult<ComponentStorage> {
    let mut storage = ComponentStorage::new();
    storage.register_component::<SpriteTag>()?;
    storage.register_component::<PlayerTag>()?;
    storage.register_component::<NpcTag>()?;
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
    Ok(storage)
}
