use components::SoundFx;
use engine::{
    fetch_first, game_scene::SceneParameters, systems::GameSystem, ComponentStorage, EngineError,
    EngineResult, EntityID, Float, Query, Vec2f,
};

use crate::{
    game_scene::subsystems::{can_shoot, get_actor_state, update_weapon_state},
    resource::{
        NPC_SOLDIER_ATTACK, NPC_SOLDIER_DAMAGE, NPC_SOLDIER_DEATH, NPC_SOLDIER_IDLE,
        NPC_SOLDIER_WALK, SCENE_MAIN_MENU, SCENE_PARAM_WIN, SOUND_NPC_ATTACK, SOUND_NPC_DEATH,
        SOUND_NPC_PAIN,
    },
};

use super::{
    components::{self, ActorState, Sprite},
    subsystems::{
        fetch_player_id, is_actor_dead, ray_cast_from_entity, replace_actor_state, updated_state,
    },
};

pub const NPC_SOLDIER_SHOT_DEADLINE: usize = 10;
pub const NPC_SOLDIER_DAMAGE_RECOVER: usize = 20;
pub const NPC_VISION_SENSITIVITY: Float = 0.7;

#[derive(Default)]
pub struct NpcSystem {
    player_id: EntityID,
    maze_id: EntityID,
    // short term cache
    player_position: Vec2f,
    frames: usize,
    delta_time: f32,
}

impl NpcSystem {
    pub fn new() -> Self {
        Default::default()
    }

    fn update_storage_cache(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        if storage.is_alive(self.player_id) {
            return Ok(());
        }
        self.player_id = fetch_player_id(storage).ok_or(EngineError::unexpected_state(
            "[v2.npc] player entity not found",
        ))?;
        self.maze_id = fetch_first::<components::Maze>(storage).ok_or(
            EngineError::unexpected_state("[v2.npc] maze entity not found"),
        )?;
        Ok(())
    }

    fn prefetch(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        self.player_position = storage
            .get::<components::Position>(self.player_id)
            .map(|x| x.0)
            .ok_or(EngineError::component_not_found("[v2.player] Velocity"))?;
        Ok(())
    }

    fn update_npc(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        let mut state = updated_state(self.frames, storage, entity_id, NPC_SOLDIER_DAMAGE_RECOVER)?;
        if state.is_none() {
            state = self.updated_npc_action_state(storage, entity_id)?;
        }
        if let Some(new_state) = state {
            if matches!(new_state, ActorState::Dead(_)) {
                storage.set::<components::NpcTag>(entity_id, None);
                storage.set::<components::BoundingBox>(entity_id, None);
                storage.set::<components::Angle>(entity_id, None);
            }
            storage.set(entity_id, Some(new_state));
            self.update_npc_view(storage, entity_id, &new_state)?;
            self.update_npc_sound(storage, entity_id, &new_state)?;
        }

        _ = update_weapon_state(self.frames, storage, entity_id);
        let state = get_actor_state(storage, entity_id);
        let Some(angle) = storage.get::<components::Angle>(entity_id).map(|x| x.0) else {
            return Ok(());
        };
        use components::ActorState::*;
        match state {
            Walk(_) => {
                let Some(velocity) = storage.get::<components::Velocity>(entity_id).map(|x| x.0)
                else {
                    return Ok(());
                };
                let sin_a = angle.sin();
                let cos_a = angle.cos();
                let dist = velocity * self.delta_time;
                let movement = components::Movement {
                    x: dist * cos_a,
                    y: dist * sin_a,
                    angle: 0.0,
                };
                storage.set(entity_id, Some(movement));
            }
            Attack(_) => {
                if !can_shoot(storage, entity_id) {
                    return Ok(());
                }
                let Some(position) = storage.get::<components::Position>(entity_id).map(|x| x.0)
                else {
                    return Ok(());
                };
                let shot = components::Shot {
                    position,
                    angle,
                    deadline: self.frames + NPC_SOLDIER_SHOT_DEADLINE,
                };
                storage.set(entity_id, Some(shot));
                storage.set(entity_id, Some(SoundFx::once(SOUND_NPC_ATTACK)));
            }
            Idle(_) => {
                // TODO: path finding...
            }
            _ => {
                // no op
            }
        }
        Ok(())
    }

    fn updated_npc_action_state(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<Option<components::ActorState>> {
        let cur_state = get_actor_state(storage, entity_id);
        match cur_state {
            ActorState::Idle(_) | ActorState::Attack(_) | ActorState::Walk(_) => {
                self.ncp_find_target(storage, entity_id)
            }
            _ => Ok(None),
        }
    }

    fn ncp_find_target(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<Option<components::ActorState>> {
        let Some(npc_position) = storage.get::<components::Position>(entity_id).map(|x| x.0) else {
            return Ok(None);
        };
        let vector = self.player_position - npc_position;
        let angle = vector.y.atan2(vector.x);
        storage.set(entity_id, Some(components::Angle(angle)));
        let target_id = ray_cast_from_entity(
            entity_id,
            storage,
            self.maze_id,
            npc_position,
            angle,
            NPC_VISION_SENSITIVITY,
        )?;
        let new_state = match target_id {
            Some(id) if self.player_id == id => {
                if vector.hypotenuse() < 5.0 {
                    components::ActorState::Attack(usize::MAX)
                } else {
                    components::ActorState::Walk(usize::MAX)
                }
            }
            _ => ActorState::Idle(usize::MAX),
        };
        Ok(replace_actor_state(new_state, storage, entity_id))
    }

    fn update_npc_view(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
        state: &components::ActorState,
    ) -> EngineResult<()> {
        let animation = match state {
            ActorState::Undefined => None,
            ActorState::Idle(_) => Some(Sprite::with_animation(
                NPC_SOLDIER_IDLE,
                self.frames,
                usize::MAX,
            )),
            ActorState::Dead(_) => Some(Sprite::with_animation(NPC_SOLDIER_DEATH, self.frames, 1)),
            ActorState::Walk(_) => Some(Sprite::with_animation(
                NPC_SOLDIER_WALK,
                self.frames,
                usize::MAX,
            )),
            ActorState::Attack(_) => Some(Sprite::with_animation(
                NPC_SOLDIER_ATTACK,
                self.frames,
                usize::MAX,
            )),
            ActorState::Damaged(_) => Some(Sprite::with_animation(
                NPC_SOLDIER_DAMAGE,
                self.frames,
                usize::MAX,
            )),
        };
        storage.set(entity_id, animation);
        Ok(())
    }

    fn update_npc_sound(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
        state: &components::ActorState,
    ) -> EngineResult<()> {
        let sound_fx = match state {
            ActorState::Dead(_) => Some(SoundFx::once(SOUND_NPC_DEATH)),
            ActorState::Damaged(_) => Some(SoundFx::once(SOUND_NPC_PAIN)),
            _ => None,
        };
        storage.set(entity_id, sound_fx);
        Ok(())
    }
}

impl GameSystem for NpcSystem {
    fn setup(
        &mut self,
        storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        self.update_storage_cache(storage)?;
        println!("[v2.npc] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        frames: usize,
        delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<engine::systems::GameSystemCommand> {
        self.update_storage_cache(storage)?;
        self.prefetch(storage)?;
        self.frames = frames;
        self.delta_time = delta_time;

        let query = Query::new().with_component::<components::NpcTag>();
        let entities = storage.fetch_entities(&query);
        let mut alive_npc = false;
        for entity_id in entities {
            self.update_npc(storage, entity_id)?;
            alive_npc |= !is_actor_dead(storage, entity_id);
        }

        let command = if alive_npc {
            engine::systems::GameSystemCommand::Nothing
        } else {
            let mut params = SceneParameters::default();
            params.insert(SCENE_PARAM_WIN.to_string(), "".to_string());
            engine::systems::GameSystemCommand::SwitchScene {
                id: SCENE_MAIN_MENU,
                params,
            }
        };
        Ok(command)
    }
}
