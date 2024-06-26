use engine::{
    game_scene::SceneParameters,
    systems::{GameSystem, GameSystemCommand},
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityID, Float,
};

use crate::{
    game_scene::{components::Sprite, subsystems::update_weapon_state},
    resource::{
        PLAYER_SHOTGUN_IDLE_ANIM, PLAYER_SHOTGUN_SHOT_ANIM, SCENE_MAIN_MENU, SCENE_PARAM_PAUSE,
        SOUND_PLAYER_ATTACK, SOUND_PLAYER_PAIN, WORLD_GAME_OVER,
    },
};

use super::{
    components::{self, ControllerState, Movement, Shot},
    subsystems::{can_shoot, fetch_player_id, is_actor_dead, updated_state},
};

pub const PLAYER_SHOT_DEADLINE: usize = 3;
pub const PLAYER_DAMAGE_DAMAGE_RECOVER: usize = 5;

struct InputResult {
    pub movement: Movement,
    pub command: GameSystemCommand,
    pub is_shooting: bool,
}

impl Default for InputResult {
    fn default() -> Self {
        Self {
            movement: Default::default(),
            command: GameSystemCommand::Nothing,
            is_shooting: Default::default(),
        }
    }
}

#[derive(Default)]
pub struct PlayerSystem {
    player_id: EntityID,
    // short cache
    velocity: Float,
    angle: Float,
    rotation_speed: Float,
    frames: usize,
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

    fn prefetch(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        self.velocity = storage
            .get::<components::Velocity>(self.player_id)
            .map(|x| x.0)
            .ok_or(EngineError::component_not_found("[v2.player] Velocity"))?;
        self.angle = storage
            .get::<components::Angle>(self.player_id)
            .map(|x| x.0)
            .ok_or(EngineError::component_not_found("[v2.player] Angle"))?;
        self.rotation_speed = storage
            .get::<components::RotationSpeed>(self.player_id)
            .map(|x| x.0)
            .ok_or(EngineError::component_not_found(
                "[v2.player] RotationSpeed",
            ))?;
        Ok(())
    }

    fn update_movement(
        &self,
        delta_time: Float,
        controller: &ControllerState,
    ) -> engine::EngineResult<components::Movement> {
        let sin_a = self.angle.sin();
        let cos_a = self.angle.cos();

        let dist = self.velocity * delta_time;
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
            rotation = -self.rotation_speed * delta_time;
        }
        if controller.rotate_right_pressed {
            rotation = self.rotation_speed * delta_time;
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
    ) -> EngineResult<InputResult> {
        let mut result = InputResult::default();
        let Some(controller) = storage.get::<components::ControllerState>(self.player_id) else {
            println!("[v2.player] warn: controller component isn't associated with player");
            return Ok(result);
        };
        if controller.pause_pressed {
            let mut params = SceneParameters::default();
            params.insert(SCENE_PARAM_PAUSE.to_string(), "".to_string());
            result.command = GameSystemCommand::SwitchScene {
                id: SCENE_MAIN_MENU,
                params,
            };
        }
        result.is_shooting = controller.shot_pressed;
        result.movement = self.update_movement(delta_time, &controller)?;
        Ok(result)
    }

    fn update_weapon(&self, storage: &mut ComponentStorage) -> EngineResult<()> {
        use components::WeaponState::*;
        let Some(state) = update_weapon_state(self.frames, storage, self.player_id) else {
            return Ok(());
        };
        let sprite = match state {
            Undefined => None,
            Recharge(_) => Some(Sprite::with_animation(
                PLAYER_SHOTGUN_SHOT_ANIM,
                self.frames,
                1,
            )),
            Ready(_) => Some(Sprite::with_animation(
                PLAYER_SHOTGUN_IDLE_ANIM,
                self.frames,
                usize::MAX,
            )),
        };
        storage.set(self.player_id, sprite);
        Ok(())
    }

    fn handle_shot(&self, storage: &mut ComponentStorage) -> EngineResult<()> {
        if !can_shoot(storage, self.player_id) {
            return Ok(());
        }
        let Some(position) = storage
            .get::<components::Position>(self.player_id)
            .map(|x| x.0)
        else {
            return Ok(());
        };
        let shot = Shot {
            angle: self.angle,
            position,
            deadline: self.frames + PLAYER_SHOT_DEADLINE,
        };
        storage.set(self.player_id, Some(shot));

        let sound_fx = components::SoundFx::once(SOUND_PLAYER_ATTACK);
        storage.set(self.player_id, Some(sound_fx));

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
        self.prefetch(storage)?;
        self.frames = frames;

        if let Some(new_state) = updated_state(
            self.frames,
            storage,
            self.player_id,
            PLAYER_DAMAGE_DAMAGE_RECOVER,
        )? {
            storage.set(self.player_id, Some(new_state));
            match new_state {
                components::ActorState::Dead(_) => {
                    storage.set::<components::BoundingBox>(self.player_id, None);
                    let sprite = Sprite::with_texture(WORLD_GAME_OVER);
                    storage.set(self.player_id, Some(sprite));
                }
                components::ActorState::Damaged(_) => {
                    let sound_fx = components::SoundFx::once(SOUND_PLAYER_PAIN);
                    storage.set(self.player_id, Some(sound_fx));
                }
                _ => {
                    // no op
                }
            }
        }

        let input = self.handle_controls(delta_time, storage)?;
        if !is_actor_dead(storage, self.player_id) {
            storage.set(self.player_id, Some(input.movement));
            if input.is_shooting {
                self.handle_shot(storage)?;
            }
            self.update_weapon(storage)?;
        }
        Ok(input.command)
    }
}
