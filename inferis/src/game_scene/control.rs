use std::borrow::BorrowMut;

use engine::{
    keyboard::Keycode, systems::GameControlSystem, EngineError, EngineResult, EntityID, InputEvent,
};

use crate::game_scene::fetch_player_id;

use super::components;

#[derive(Default)]
pub struct ControlSystem {
    player_id: EntityID,
}

impl ControlSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl GameControlSystem for ControlSystem {
    fn setup(&mut self, storage: &engine::ComponentStorage) -> EngineResult<()> {
        let Some(player_id) = fetch_player_id(storage) else {
            return Err(EngineError::unexpected_state(
                "[v2.controller] player entity not found",
            ));
        };
        self.player_id = player_id;
        println!("[v2.controller] setup ok");
        Ok(())
    }

    fn push_events(
        &mut self,
        storage: &mut engine::ComponentStorage,
        events: &[engine::InputEvent],
    ) -> EngineResult<()> {
        let Some(mut comp) = storage.get_mut::<components::ControllerState>(self.player_id) else {
            println!("[v2.controller] warn: controller component isn't associated with player");
            return Ok(());
        };
        let state = comp.borrow_mut();
        state.mouse_x_relative = 0;
        state.mouse_y_relative = 0;
        for event in events {
            match event {
                InputEvent::Keyboard { code, pressed } => {
                    use Keycode::*;
                    match code {
                        Up | W => state.forward_pressed = *pressed,
                        Down | S => state.backward_pressed = *pressed,
                        A => state.left_pressed = *pressed,
                        D => state.right_pressed = *pressed,
                        Left => state.rotate_left_pressed = *pressed,
                        Right => state.rotate_right_pressed = *pressed,
                        X => state.shot_pressed = *pressed,
                        Escape => state.exit_pressed = *pressed,
                        _ => {
                            // println!("Key {code} pressed {pressed}")
                        }
                    }
                }
                InputEvent::Mouse { x_rel, y_rel, .. } => {
                    state.mouse_x_relative = *x_rel;
                    state.mouse_y_relative = *y_rel;
                }
                InputEvent::Quit => state.exit_pressed = true,
            }
        }
        Ok(())
    }
}
