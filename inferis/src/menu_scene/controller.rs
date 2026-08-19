use engine::{
    prelude::Keycode,
    refresh_cached_entity,
    systems::{GameControlSystem, InputEvent},
    EngineResult, EntityID,
};
use log::{info, warn};

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
        refresh_cached_entity::<CursorTag>(
            storage,
            &mut self.cursor_id,
            "[v2.menu.controller] cursor",
        )
    }
}

impl GameControlSystem for MenuControlSystem {
    fn setup(&mut self, storage: &engine::ComponentStorage) -> engine::EngineResult<()> {
        self.update_storage_cache(storage)?;
        info!("setup ok");
        Ok(())
    }

    fn push_events(
        &mut self,
        storage: &mut engine::ComponentStorage,
        events: &[engine::systems::InputEvent],
    ) -> engine::EngineResult<()> {
        self.update_storage_cache(storage)?;
        let Some(mut comp) = storage.get_mut::<components::ControllerState>(self.cursor_id) else {
            warn!("controller component isn't associated with cursor");
            return Ok(());
        };
        let state = &mut *comp;
        for event in events {
            let InputEvent::Keyboard { code, pressed } = event else {
                continue;
            };
            match *code {
                Keycode::UP => state.up_pressed = *pressed,
                Keycode::DOWN => state.down_pressed = *pressed,
                Keycode::RETURN => state.select_pressed = *pressed,
                _ => {
                    // no op
                }
            }
        }
        Ok(())
    }
}
