use engine::world::InputEvent;

const KEYCODE_W: i32 = 119;
const KEYCODE_S: i32 = 115;
const KEYCODE_A: i32 = 97;
const KEYCODE_D: i32 = 100;
const KEYCODE_LEFT: i32 = 1073741904;
const KEYCODE_RIGHT: i32 = 1073741903;
const KEYCODE_UP: i32 = 1073741906;
const KEYCODE_DOWN: i32 = 1073741905;

#[derive(Default)]
pub struct ControllerState {
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
        for event in events {
            match event {
                InputEvent::Keyboard { code, pressed } => {
                    self.handle_key(*code, *pressed);
                }
                InputEvent::Mouse { x, y, x_rel, y_rel } => {
                    // println!("Mouse x = {x}, dx = {x_rel}, y = {y}, dy = {y_rel}");
                }
            }
        }
    }

    fn handle_key(&mut self, code: i32, pressed: bool) {
        match code {
            KEYCODE_UP | KEYCODE_W => self.forward_pressed = pressed,
            KEYCODE_DOWN | KEYCODE_S => self.backward_pressed = pressed,
            KEYCODE_A => self.left_pressed = pressed,
            KEYCODE_D => self.right_pressed = pressed,
            KEYCODE_LEFT => self.rotate_left_pressed = pressed,
            KEYCODE_RIGHT => self.rotate_right_pressed = pressed,
            _ => {
                // println!("Key {code} pressed {pressed}")
            }
        }
    }

    pub fn clean_up(&mut self) {
        self.mouse_x_relative = 0;
        self.mouse_y_relative = 0;
    }
}
