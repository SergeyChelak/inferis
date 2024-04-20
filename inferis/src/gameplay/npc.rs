use engine::{ComponentStorage, EngineResult, EntityID, Query};

use crate::resource::*;

use super::{AnimationData, NpcDisplayMode, NpcTag};

#[derive(Clone, Copy, Debug)]
pub enum State {
    Idle,
    Death,
    Attack,
    Walk,
    Damage,
}

pub fn npc_update(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query: Query = Query::new().with_component::<NpcTag>();
    for entity_id in storage.fetch_entities(&query) {
        update_animation(storage, entity_id)?;
    }
    Ok(())
}

fn update_animation(storage: &mut ComponentStorage, entity_id: EntityID) -> EngineResult<()> {
    let Some(state) = storage.get::<NpcDisplayMode>(entity_id).map(|x| x.0) else {
        storage.set::<AnimationData>(entity_id, None);
        return Ok(());
    };
    let new_animation_data = animation_data(state);
    if storage.get::<AnimationData>(entity_id).is_none() {
        storage.set::<AnimationData>(entity_id, Some(new_animation_data));
        return Ok(());
    };
    if let Some(animation) = storage.get::<AnimationData>(entity_id).and_then(|x| {
        if new_animation_data.animation_id == x.animation_id {
            None
        } else {
            Some(new_animation_data)
        }
    }) {
        storage.set::<AnimationData>(entity_id, Some(animation));
    }
    Ok(())
}

fn animation_data(state: State) -> AnimationData {
    use State::*;
    match state {
        Idle => AnimationData::endless(NPC_SOLDIER_IDLE),
        Death => AnimationData::new(NPC_SOLDIER_DEATH, 90),
        Attack => AnimationData::endless(NPC_SOLDIER_ATTACK),
        Walk => AnimationData::endless(NPC_SOLDIER_WALK),
        Damage => AnimationData::new(NPC_SOLDIER_DAMAGE, 10),
    }
}
