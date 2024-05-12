use components::SoundFx;
use engine::{
    systems::GameSystem, ComponentStorage, EngineError, EngineResult, EntityID, Query, Vec2f,
};

use crate::{
    gameplay::{BoundingBox, NpcTag},
    resource::{
        NPC_SOLDIER_ATTACK, NPC_SOLDIER_DAMAGE, NPC_SOLDIER_DAMAGE_RECOVER, NPC_SOLDIER_DEATH,
        NPC_SOLDIER_IDLE, NPC_SOLDIER_WALK, SOUND_NPC_ATTACK, SOUND_NPC_DEATH, SOUND_NPC_PAIN,
    },
};

use super::{
    components::{self, ActorState, Sprite},
    fetch_player_id,
    subsystems::updated_state,
};

#[derive(Default)]
pub struct NpcSystem {
    player_id: EntityID,
    // short term cache
    player_position: Vec2f,
    frames: usize,
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
        if let Some(new_state) =
            updated_state(self.frames, storage, entity_id, NPC_SOLDIER_DAMAGE_RECOVER)?
        {
            if matches!(new_state, ActorState::Dead(_)) {
                storage.set::<NpcTag>(entity_id, None);
                storage.set::<BoundingBox>(entity_id, None);
            }
            storage.set(entity_id, Some(new_state));
            self.update_npc_view(storage, entity_id, &new_state)?;
            self.update_npc_sound(storage, entity_id, &new_state)?;
        }
        Ok(())
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
            ActorState::Dead(_) => Some(Sprite::with_animation(NPC_SOLDIER_DEATH, self.frames, 0)),
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
            ActorState::Attack(_) => Some(SoundFx::once(SOUND_NPC_ATTACK)),
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
        _delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<engine::systems::GameSystemCommand> {
        self.update_storage_cache(storage)?;
        self.prefetch(storage)?;
        self.frames = frames;

        let query = Query::new().with_component::<components::NpcTag>();
        let entities = storage.fetch_entities(&query);
        for entity_id in entities {
            self.update_npc(storage, entity_id)?;
        }
        Ok(engine::systems::GameSystemCommand::Nothing)
    }
}
