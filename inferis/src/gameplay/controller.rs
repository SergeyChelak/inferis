use engine::{keyboard::Keycode, world::InputEvent};

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
}

impl ControllerState {
    pub fn update(&mut self, events: &[InputEvent]) {
        self.reset_relative();
        for event in events {
            match event {
                InputEvent::Keyboard { code, pressed } => {
                    self.handle_key(*code, *pressed);
                }
                InputEvent::Mouse { x_rel, y_rel, .. } => {
                    self.mouse_x_relative = *x_rel;
                    self.mouse_y_relative = *y_rel;
                }
            }
        }
    }

    fn handle_key(&mut self, code: Keycode, pressed: bool) {
        use Keycode::*;
        match code {
            Up | W => self.forward_pressed = pressed,
            Down | S => self.backward_pressed = pressed,
            A => self.left_pressed = pressed,
            D => self.right_pressed = pressed,
            Left => self.rotate_left_pressed = pressed,
            Right => self.rotate_right_pressed = pressed,
            X => self.shot_pressed = pressed,
            _ => {
                // println!("Key {code} pressed {pressed}")
            }
        }
    }

    fn reset_relative(&mut self) {
        self.mouse_x_relative = 0;
        self.mouse_y_relative = 0;
    }
}
