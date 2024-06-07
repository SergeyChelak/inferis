use std::borrow::BorrowMut;

use engine::{
    keyboard::Keycode,
    systems::{GameControlSystem, InputEvent},
    ComponentStorage, EngineError, EngineResult, EntityID,
};

use super::{components, subsystems::fetch_player_id};

#[derive(Default)]
pub struct ControlSystem {
    player_id: EntityID,
}

impl ControlSystem {
    pub fn new() -> Self {
        Default::default()
    }

    fn update_storage_cache(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        if storage.is_alive(self.player_id) {
            return Ok(());
        }
        self.player_id = fetch_player_id(storage).ok_or(EngineError::unexpected_state(
            "[v2.controller] player entity not found",
        ))?;
        Ok(())
    }
}

impl GameControlSystem for ControlSystem {
    fn setup(&mut self, storage: &engine::ComponentStorage) -> EngineResult<()> {
        self.update_storage_cache(storage)?;
        println!("[v2.controller] setup ok");
        Ok(())
    }

    fn push_events(
        &mut self,
        storage: &mut engine::ComponentStorage,
        events: &[InputEvent],
    ) -> EngineResult<()> {
        self.update_storage_cache(storage)?;
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
                        Escape => state.pause_pressed = *pressed,
                        _ => {
                            // println!("Key {code} pressed {pressed}")
                        }
                    }
                }
                InputEvent::Mouse { x_rel, y_rel, .. } => {
                    state.mouse_x_relative = *x_rel;
                    state.mouse_y_relative = *y_rel;
                }
                InputEvent::Quit => state.pause_pressed = true,
            }
        }
        Ok(())
    }
}
