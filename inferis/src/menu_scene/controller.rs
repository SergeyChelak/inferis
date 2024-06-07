use std::borrow::BorrowMut;

use engine::{
    fetch_first,
    keyboard::Keycode,
    systems::{GameControlSystem, InputEvent},
    EngineError, EngineResult, EntityID,
};

use super::components::{self, CursorTag};

pub struct MenuControlSystem {
    cursor_id: EntityID,
}

impl MenuControlSystem {
    pub fn new() -> Self {
        Self {
            cursor_id: Default::default(),
        }
    }

    fn update_storage_cache(&mut self, storage: &engine::ComponentStorage) -> EngineResult<()> {
        if storage.is_alive(self.cursor_id) {
            return Ok(());
        }
        self.cursor_id = fetch_first::<CursorTag>(storage).ok_or(EngineError::unexpected_state(
            "[v2.menu.controller] cursor entity not found",
        ))?;
        Ok(())
    }
}

impl GameControlSystem for MenuControlSystem {
    fn setup(&mut self, storage: &engine::ComponentStorage) -> engine::EngineResult<()> {
        self.update_storage_cache(storage)?;
        println!("[v2.menu.controller] setup ok");
        Ok(())
    }

    fn push_events(
        &mut self,
        storage: &mut engine::ComponentStorage,
        events: &[engine::systems::InputEvent],
    ) -> engine::EngineResult<()> {
        self.update_storage_cache(storage)?;
        let Some(mut comp) = storage.get_mut::<components::ControllerState>(self.cursor_id) else {
            println!(
                "[v2.menu.controller] warn: controller component isn't associated with player"
            );
            return Ok(());
        };
        let state = comp.borrow_mut();
        for event in events {
            let InputEvent::Keyboard { code, pressed } = event else {
                continue;
            };
            use Keycode::*;
            match code {
                Up | W => state.up_pressed = *pressed,
                Down | S => state.down_pressed = *pressed,
                X | KpEnter => state.select_pressed = *pressed,
                _ => {
                    // no op
                }
            }
        }
        Ok(())
    }
}
