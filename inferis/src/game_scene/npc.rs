use components::SoundFx;
use engine::{
    game_scene::SceneParameters, refresh_cached_entity, systems::GameSystem, ComponentStorage,
    EngineError, EngineResult, EntityID, Float, Query, Vec2f,
};
use log::info;

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
    subsystems::{is_actor_dead, ray_cast_from_entity, replace_actor_state, updated_state},
};

pub const NPC_SOLDIER_SHOT_DEADLINE: usize = 10;
pub const NPC_SOLDIER_DAMAGE_RECOVER: usize = 20;
/// Distance at which a soldier stops closing in and starts shooting.
const NPC_SOLDIER_ATTACK_DISTANCE: Float = 5.0;

/// How many simulation steps a soldier may reuse its last line-of-sight
/// verdict for, chosen by its distance to the player: `(distance below,
/// steps between casts)`, first match wins.
///
/// The cast is the dominant per-step AI cost, and the further away the
/// player is the longer a stale verdict goes unnoticed -- a soldier twenty
/// tiles off that keeps walking for another fifth of a second after the
/// player breaks line of sight looks no different. Inside attack range the
/// cast still runs every step, where reaction time is actually visible.
const TARGET_CHECK_INTERVALS: [(Float, usize); 3] = [
    (NPC_SOLDIER_ATTACK_DISTANCE * 2.0, 1),
    (15.0, 4),
    (Float::INFINITY, 12),
];

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
        refresh_cached_entity::<components::PlayerTag>(
            storage,
            &mut self.player_id,
            "[v2.npc] player",
        )?;
        refresh_cached_entity::<components::Maze>(storage, &mut self.maze_id, "[v2.npc] maze")
    }

    fn prefetch(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        self.player_position = storage
            .get::<components::Position>(self.player_id)
            .map(|x| x.0)
            .ok_or(EngineError::component_not_found("[v2.npc] player Position"))?;
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
                storage.set::<components::NpcTag>(entity_id, None)?;
                storage.set::<components::BoundingBox>(entity_id, None)?;
                storage.set::<components::Angle>(entity_id, None)?;
            }
            storage.set(entity_id, Some(new_state))?;
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
                storage.set(entity_id, Some(movement))?;
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
                storage.set(entity_id, Some(shot))?;
                storage.set(entity_id, Some(SoundFx::once(SOUND_NPC_ATTACK)))?;
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
        let distance = vector.length();
        // turning to face the player is cheap and keeps the chase smooth,
        // so it stays on every step even when the cast below does not
        let angle = vector.y.atan2(vector.x);
        storage.set(entity_id, Some(components::Angle(angle)))?;

        if !self.is_target_check_due(entity_id, distance) {
            // no verdict this step: the soldier keeps acting on the last one
            return Ok(None);
        }

        let target_id =
            ray_cast_from_entity(entity_id, storage, self.maze_id, npc_position, angle)?;
        let new_state = match target_id {
            Some(id) if self.player_id == id => {
                if distance < NPC_SOLDIER_ATTACK_DISTANCE {
                    components::ActorState::Attack(usize::MAX)
                } else {
                    components::ActorState::Walk(usize::MAX)
                }
            }
            _ => ActorState::Idle(usize::MAX),
        };
        replace_actor_state(new_state, storage, entity_id)
    }

    /// Whether this soldier should re-cast toward the player on this step.
    ///
    /// The entity index offsets the phase so that soldiers sharing an
    /// interval spread their casts across it rather than all firing on the
    /// same step and spiking one frame.
    fn is_target_check_due(&self, entity_id: EntityID, distance: Float) -> bool {
        let interval = target_check_interval(distance);
        interval <= 1 || (self.frames + entity_id.index()).is_multiple_of(interval)
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
        storage.set(entity_id, animation)?;
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
        storage.set(entity_id, sound_fx)?;
        Ok(())
    }
}

/// Steps between line-of-sight casts for a soldier at `distance`.
fn target_check_interval(distance: Float) -> usize {
    TARGET_CHECK_INTERVALS
        .iter()
        .find(|(below, _)| distance < *below)
        .map(|(_, interval)| *interval)
        .unwrap_or(1)
}

impl GameSystem for NpcSystem {
    fn setup(
        &mut self,
        storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        self.update_storage_cache(storage)?;
        info!("setup ok");
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn inside_attack_range_the_cast_runs_every_step() {
        assert_eq!(target_check_interval(0.0), 1);
        assert_eq!(target_check_interval(NPC_SOLDIER_ATTACK_DISTANCE), 1);
        assert_eq!(target_check_interval(9.9), 1);
    }

    #[test]
    fn the_further_away_the_rarer_the_cast() {
        assert_eq!(target_check_interval(10.0), 4);
        assert_eq!(target_check_interval(14.9), 4);
        assert_eq!(target_check_interval(15.0), 12);
        assert_eq!(target_check_interval(1000.0), 12);
    }

    #[test]
    fn no_interval_is_zero() {
        // a zero interval would make every step look "not due", freezing the
        // soldier on whatever verdict it happened to hold
        assert!(TARGET_CHECK_INTERVALS.iter().all(|(_, steps)| *steps > 0));
    }

    #[test]
    fn soldiers_sharing_an_interval_spread_their_casts_across_it() {
        let mut storage = ComponentStorage::new();
        let ids = (0..4).map(|_| storage.add_entity()).collect::<Vec<_>>();
        let distance = 12.0;
        let interval = target_check_interval(distance);
        assert_eq!(interval, 4);

        let mut system = NpcSystem::new();
        // each soldier is due exactly once per interval ...
        for id in &ids {
            let mut due = 0;
            for step in 0..interval {
                system.frames = step;
                if system.is_target_check_due(*id, distance) {
                    due += 1;
                }
            }
            assert_eq!(due, 1, "entity {} is due {} times", id.index(), due);
        }
        // ... and no single step carries more than its share
        for step in 0..interval {
            system.frames = step;
            let due = ids
                .iter()
                .filter(|id| system.is_target_check_due(**id, distance))
                .count();
            assert_eq!(due, 1, "step {step} carries {due} casts");
        }
    }
}
