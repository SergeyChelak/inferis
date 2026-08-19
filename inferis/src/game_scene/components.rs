use super::generator;
use crate::resource::*;
use engine::{Float, SizeFloat, Vec2f};
use std::{collections::HashSet, fmt::Display};

pub struct PlayerTag;
pub struct NpcTag;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ActorState {
    #[default]
    Undefined,
    Idle(usize),
    Dead(usize),
    Attack(usize),
    Walk(usize),
    Damaged(usize),
}

impl Display for ActorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorState::Undefined => write!(f, "Undefined")?,
            ActorState::Idle(deadline) => write!(f, "Idle @ {deadline}")?,
            ActorState::Dead(deadline) => write!(f, "Dead @ {deadline}")?,
            ActorState::Attack(deadline) => write!(f, "Attack @ {deadline}")?,
            ActorState::Walk(deadline) => write!(f, "Walk @ {deadline}")?,
            ActorState::Damaged(deadline) => write!(f, "Damaged @ {deadline}")?,
        }
        Ok(())
    }
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
    pub pause_pressed: bool,
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

#[derive(Clone, Copy)]
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

pub struct Maze {
    pub matrix: generator::matrix::Matrix,
    pub contour: HashSet<generator::matrix::Position>,
}

/// Wall texture per matrix value: the maze stores `0` for floor and
/// `1..=WALL_TEXTURES.len()` for walls, so value `n` picks index `n - 1`.
pub const WALL_TEXTURES: [&str; 5] = [
    WORLD_WALL1,
    WORLD_WALL2,
    WORLD_WALL3,
    WORLD_WALL4,
    WORLD_WALL5,
];

impl Maze {
    pub fn wall_texture(&self, point: Vec2f) -> Option<&'static str> {
        let index = self.value_at(point)?.checked_sub(1)?;
        WALL_TEXTURES.get(usize::try_from(index).ok()?).copied()
    }

    pub fn value_at(&self, point: Vec2f) -> Option<&i32> {
        let Vec2f { x, y } = point;
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let (col, row) = (point.x as usize, point.y as usize);
        self.matrix.get(row).and_then(|x| x.get(col))
    }

    /// Ray steps needed to cross the maze from any tile in any direction.
    ///
    /// A ray advances one row or one column per step, so the larger of the
    /// two dimensions always reaches the far side. Passing this to
    /// `ray_cast` is what keeps the caster in step with the maze size: a
    /// bigger maze automatically gets a longer reach.
    pub fn ray_cast_steps(&self) -> usize {
        let rows = self.matrix.len();
        let cols = self.matrix.first().map(|row| row.len()).unwrap_or_default();
        rows.max(cols)
    }

    pub fn is_wall(&self, point: Vec2f) -> bool {
        let Some(val) = self.value_at(point) else {
            return true;
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

#[cfg(test)]
mod test {
    use super::*;

    fn maze(matrix: generator::matrix::Matrix) -> Maze {
        Maze {
            matrix,
            contour: HashSet::new(),
        }
    }

    #[test]
    fn wall_texture_maps_matrix_value_to_texture() {
        let maze = maze(vec![vec![0, 1, 2, 3, 4, 5, 6]]);
        // 0 is floor, values past the last texture have none either
        assert_eq!(maze.wall_texture(Vec2f::new(0.0, 0.0)), None);
        assert_eq!(maze.wall_texture(Vec2f::new(6.0, 0.0)), None);
        for (col, expected) in WALL_TEXTURES.iter().enumerate() {
            let point = Vec2f::new(col as Float + 1.0, 0.0);
            assert_eq!(maze.wall_texture(point), Some(*expected));
        }
    }

    #[test]
    fn ray_cast_steps_covers_the_longer_side() {
        // a ray advances one row or one column per step, so the bound has to
        // be the larger dimension for the short axis not to cut it short
        assert_eq!(maze(vec![vec![0; 30]; 12]).ray_cast_steps(), 30);
        assert_eq!(maze(vec![vec![0; 12]; 30]).ray_cast_steps(), 30);
        assert_eq!(maze(vec![]).ray_cast_steps(), 0);
    }

    #[test]
    fn wall_texture_outside_the_matrix_is_none() {
        let maze = maze(vec![vec![1]]);
        assert_eq!(maze.wall_texture(Vec2f::new(-1.0, 0.0)), None);
        assert_eq!(maze.wall_texture(Vec2f::new(0.0, -1.0)), None);
        assert_eq!(maze.wall_texture(Vec2f::new(1.0, 0.0)), None);
        assert_eq!(maze.wall_texture(Vec2f::new(0.0, 1.0)), None);
    }
}
