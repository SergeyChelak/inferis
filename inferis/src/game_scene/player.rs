use engine::{
    systems::{GameSystem, GameSystemCommand},
    AssetManager, ComponentStorage, EngineError, EntityID, Float,
};

use crate::game_scene::fetch_player_id;

use super::components::{self, ControllerState, Movement};

#[derive(Default)]
pub struct PlayerSystem {
    player_id: EntityID,
}

impl PlayerSystem {
    pub fn new() -> Self {
        Default::default()
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
        Ok(calculate_movement(
            &controller,
            delta_time,
            velocity,
            rotation_speed,
            angle,
        ))
    }
}

impl GameSystem for PlayerSystem {
    fn setup(
        &mut self,
        storage: &engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        let Some(player_id) = fetch_player_id(storage) else {
            return Err(EngineError::unexpected_state(
                "[v2.player] player entity not found",
            ));
        };
        self.player_id = player_id;
        println!("[v2.player] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        _frames: usize,
        delta_time: Float,
        storage: &mut ComponentStorage,
        _assets: &AssetManager,
    ) -> engine::EngineResult<GameSystemCommand> {
        let movement: Movement;
        let mut command = GameSystemCommand::Nothing;
        // put controller logic inside code block to make a borrow checker happy
        {
            let Some(controller) = storage.get::<components::ControllerState>(self.player_id)
            else {
                println!("[v2.controller] warn: controller component isn't associated with player");
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
}

fn calculate_movement(
    controller: &ControllerState,
    delta_time: f32,
    velocity: Float,
    rotation_speed: Float,
    angle: Float,
) -> Movement {
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
    components::Movement {
        x: dx,
        y: dy,
        angle: rotation,
    }
}
