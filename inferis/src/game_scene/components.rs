use engine::{Float, Vec2f};

pub struct PlayerTag;
pub struct NpcTag;
pub struct InvalidatedTag;

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

pub struct Angle(pub Float);

pub type HealthType = u32;

pub struct Health(pub HealthType);

//  texture id?
pub struct Sprite(pub &'static str);

// sprite position parameters
pub struct ScaleRatio(pub Float);
pub struct HeightShift(pub Float);

//
pub type MazeData = Vec<Vec<i32>>;
pub struct Maze(pub MazeData);
