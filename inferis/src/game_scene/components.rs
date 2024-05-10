use engine::Float;

pub struct PlayerTag;

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

pub struct Velocity(pub Float);

pub struct RotationSpeed(pub Float);

pub struct Angle(pub Float);
