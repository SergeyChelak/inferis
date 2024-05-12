use engine::{Float, SizeFloat, Vec2f};

use crate::resource::*;

pub struct PlayerTag;
pub struct NpcTag;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActorState {
    Undefined,
    Idle(usize),
    Dead(usize),
    Attack(usize),
    Walk(usize),
    Damaged(usize),
}

#[derive(Default)]
pub struct ControllerState {
    pub shot_pressed: bool,
    pub forward_pressed: bool,
    pub backward_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub rotate_left_pressed: bool,
    pub rotate_right_pressed: bool,
    pub mouse_x_relative: i32,
    pub mouse_y_relative: i32,
    pub exit_pressed: bool,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Movement {
    pub x: Float,
    pub y: Float,
    pub angle: Float,
}
pub struct Position(pub Vec2f);

pub struct Velocity(pub Float);

pub struct RotationSpeed(pub Float);

pub struct BoundingBox(pub SizeFloat);

pub struct Angle(pub Float);

pub type HealthType = u32;

pub struct Health(pub HealthType);

pub struct Shot {
    pub position: Vec2f,
    pub angle: Float,
    pub deadline: usize,
}

#[derive(Clone, Copy)]
pub struct Damage(pub HealthType);

pub enum SpriteView {
    Texture {
        asset_id: &'static str,
    },
    Animation {
        asset_id: &'static str,
        frame_start: usize,
        times: usize,
    },
}

pub struct Sprite {
    pub view: SpriteView,
}

impl Sprite {
    pub fn with_texture(asset_id: &'static str) -> Self {
        Self {
            view: SpriteView::Texture { asset_id },
        }
    }

    pub fn with_animation(asset_id: &'static str, frame_start: usize, times: usize) -> Self {
        Self {
            view: SpriteView::Animation {
                asset_id,
                frame_start,
                times,
            },
        }
    }
}

// sprite position parameters
pub struct ScaleRatio(pub Float);
pub struct HeightShift(pub Float);

//
pub type MazeData = Vec<Vec<i32>>;
pub struct Maze(pub MazeData);

impl Maze {
    pub fn wall_texture(&self, point: Vec2f) -> Option<String> {
        match self.value_at(point)? {
            1 => Some(WORLD_WALL1.to_string()),
            2 => Some(WORLD_WALL2.to_string()),
            3 => Some(WORLD_WALL3.to_string()),
            4 => Some(WORLD_WALL4.to_string()),
            5 => Some(WORLD_WALL5.to_string()),
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

pub struct SoundFx {
    pub asset_id: String,
    pub loops: i32,
}

impl SoundFx {
    pub fn once(id: impl Into<String>) -> Self {
        Self {
            asset_id: id.into(),
            loops: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Weapon {
    pub damage: HealthType,
    pub recharge_time: usize,
    pub ammo_count: usize,
    pub state: WeaponState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WeaponState {
    Undefined,
    Ready(usize),
    Recharge(usize),
}
