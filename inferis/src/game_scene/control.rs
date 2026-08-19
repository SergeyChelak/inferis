use engine::{
    prelude::Keycode,
    refresh_cached_entity,
    systems::{GameControlSystem, InputEvent},
    ComponentStorage, EngineResult, EntityID,
};
use log::{info, trace, warn};

use super::components;

#[derive(Default)]
pub struct ControlSystem {
    player_id: EntityID,
}

impl ControlSystem {
    pub fn new() -> Self {
        Default::default()
    }

    fn update_storage_cache(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        refresh_cached_entity::<components::PlayerTag>(
            storage,
            &mut self.player_id,
            "[v2.controller] player",
        )
    }
}

impl GameControlSystem for ControlSystem {
    fn setup(&mut self, storage: &engine::ComponentStorage) -> EngineResult<()> {
        self.update_storage_cache(storage)?;
        info!("setup ok");
        Ok(())
    }

    fn push_events(
        &mut self,
        storage: &mut engine::ComponentStorage,
        events: &[InputEvent],
    ) -> EngineResult<()> {
        self.update_storage_cache(storage)?;
        let Some(mut comp) = storage.get_mut::<components::ControllerState>(self.player_id) else {
            warn!("controller component isn't associated with player");
            return Ok(());
        };
        let state = &mut *comp;
        state.mouse_x_relative = 0;
        state.mouse_y_relative = 0;
        for event in events {
            match event {
                InputEvent::Keyboard { code, pressed } => match *code {
                    Keycode::UP | Keycode::W => state.forward_pressed = *pressed,
                    Keycode::DOWN | Keycode::S => state.backward_pressed = *pressed,
                    Keycode::A => state.left_pressed = *pressed,
                    Keycode::D => state.right_pressed = *pressed,
                    Keycode::LEFT => state.rotate_left_pressed = *pressed,
                    Keycode::RIGHT => state.rotate_right_pressed = *pressed,
                    Keycode::X => state.shot_pressed = *pressed,
                    Keycode::ESCAPE => state.pause_pressed = *pressed,
                    _ => trace!("unmapped key {code} pressed {pressed}"),
                },
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
