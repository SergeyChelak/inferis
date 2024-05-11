use std::borrow::BorrowMut;

use engine::{
    systems::{GameSystem, GameSystemCommand},
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityID, Float,
};

use crate::{
    game_scene::{components::Sprite, fetch_player_id},
    gameplay::Weapon,
    resource::{PLAYER_SHOTGUN_IDLE_ANIM, PLAYER_SHOTGUN_SHOT_ANIM},
};

use super::components::{self, ControllerState, Movement};

#[derive(Default)]
pub struct PlayerSystem {
    player_id: EntityID,
}

impl PlayerSystem {
    pub fn new() -> Self {
        Default::default()
    }

    fn update_storage_cache(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        if storage.is_alive(self.player_id) {
            return Ok(());
        }
        self.player_id = fetch_player_id(storage).ok_or(EngineError::unexpected_state(
            "[v2.player] player entity not found",
        ))?;
        Ok(())
    }

    fn update_movement(
        &self,
        delta_time: Float,
        storage: &ComponentStorage,
        controller: &ControllerState,
    ) -> engine::EngineResult<components::Movement> {
        let Some(velocity) = storage
            .get::<components::Velocity>(self.player_id)
            .map(|x| x.0)
        else {
            return Err(EngineError::component_not_found("[v2.player] Velocity"));
        };
        let Some(angle) = storage
            .get::<components::Angle>(self.player_id)
            .map(|x| x.0)
        else {
            return Err(EngineError::component_not_found("[v2.player] Angle"));
        };
        let Some(rotation_speed) = storage
            .get::<components::RotationSpeed>(self.player_id)
            .map(|x| x.0)
        else {
            return Err(EngineError::component_not_found(
                "[v2.player] RotationSpeed",
            ));
        };
        let sin_a = angle.sin();
        let cos_a = angle.cos();

        let dist = velocity * delta_time;
        let dist_cos = dist * cos_a;
        let dist_sin = dist * sin_a;

        let (mut dx, mut dy) = (0.0, 0.0);

        if controller.forward_pressed {
            dx += dist_cos;
            dy += dist_sin;
        }
        if controller.backward_pressed {
            dx += -dist_cos;
            dy += -dist_sin;
        }
        if controller.left_pressed {
            dx += dist_sin;
            dy += -dist_cos;
        }
        if controller.right_pressed {
            dx += -dist_sin;
            dy += dist_cos;
        }
        // rotation
        let mut rotation = 0.0;
        if controller.rotate_left_pressed {
            rotation = -rotation_speed * delta_time;
        }
        if controller.rotate_right_pressed {
            rotation = rotation_speed * delta_time;
        }
        let movement = components::Movement {
            x: dx,
            y: dy,
            angle: rotation,
        };
        Ok(movement)
    }

    fn handle_controls(
        &self,
        delta_time: Float,
        storage: &mut ComponentStorage,
    ) -> EngineResult<GameSystemCommand> {
        let movement: Movement;
        let mut command = GameSystemCommand::Nothing;
        // put controller logic inside code block to make a borrow checker happy
        {
            let Some(controller) = storage.get::<components::ControllerState>(self.player_id)
            else {
                println!("[v2.player] warn: controller component isn't associated with player");
                return Ok(GameSystemCommand::Nothing);
            };
            if controller.exit_pressed {
                command = GameSystemCommand::Terminate;
            }
            movement = self.update_movement(delta_time, storage, &controller)?;
        };
        storage.set(self.player_id, Some(movement));
        Ok(command)
    }

    fn update_weapon(&self, frames: usize, storage: &mut ComponentStorage) -> EngineResult<()> {
        use components::WeaponState::*;
        let updated;
        let state = {
            let Some(mut weapon) = storage.get_mut::<Weapon>(self.player_id) else {
                return Ok(());
            };
            use components::WeaponState::*;
            let new_state = match weapon.state {
                Undefined => Ready(usize::MAX),
                Recharge(deadline) if deadline >= frames => Ready(usize::MAX),
                state => state,
            };
            updated = new_state != weapon.state;
            if updated {
                weapon.borrow_mut().state = new_state;
            }
            new_state
        };
        if updated {
            let sprite = match state {
                Undefined => None,
                Recharge(_) => Some(Sprite::with_animation(PLAYER_SHOTGUN_SHOT_ANIM, frames, 1)),
                Ready(_) => Some(Sprite::with_animation(
                    PLAYER_SHOTGUN_IDLE_ANIM,
                    frames,
                    usize::MAX,
                )),
            };
            storage.set(self.player_id, sprite);
        }
        Ok(())
    }
}

impl GameSystem for PlayerSystem {
    fn setup(
        &mut self,
        storage: &mut ComponentStorage,
        _asset_manager: &AssetManager,
    ) -> EngineResult<()> {
        self.update_storage_cache(storage)?;
        println!("[v2.player] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        frames: usize,
        delta_time: Float,
        storage: &mut ComponentStorage,
        _asset_manager: &AssetManager,
    ) -> engine::EngineResult<GameSystemCommand> {
        self.update_storage_cache(storage)?;
        //
        self.update_weapon(frames, storage)?;
        // handle user input in the end
        self.handle_controls(delta_time, storage)
    }
}
