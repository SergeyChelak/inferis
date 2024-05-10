use engine::{
    frame_counter::AggregatedFrameCounter, game_scene::GameSystem, world::GameWorldState,
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
        world_state: &mut GameWorldState,
        _frame_counter: &mut AggregatedFrameCounter,
        delta_time: Float,
        storage: &mut ComponentStorage,
        _assets: &AssetManager,
    ) -> engine::EngineResult<Vec<engine::game_scene::Effect>> {
        let movement: Option<Movement>;
        // put all logic inside code block to make a borrow checker happy
        {
            let Some(controller) = storage.get::<components::ControllerState>(self.player_id)
            else {
                println!("[v2.controller] warn: controller component isn't associated with player");
                return Ok(vec![]);
            };
            if controller.exit_pressed {
                world_state.stop();
            }
            let Some(velocity) = storage
                .get::<components::Velocity>(self.player_id)
                .map(|x| x.0)
            else {
                return Err(EngineError::component_not_found(
                    "[handle_controls] Velocity",
                ));
            };
            let Some(angle) = storage
                .get::<components::Angle>(self.player_id)
                .map(|x| x.0)
            else {
                return Err(EngineError::component_not_found("[handle_controls] Angle"));
            };
            let Some(rotation_speed) = storage
                .get::<components::RotationSpeed>(self.player_id)
                .map(|x| x.0)
            else {
                return Err(EngineError::component_not_found(
                    "[handle_controls] RotationSpeed",
                ));
            };
            movement = Some(calculate_movement(
                &controller,
                delta_time,
                velocity,
                rotation_speed,
                angle,
            ));
        };
        storage.set(self.player_id, movement);
        Ok(vec![])
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
