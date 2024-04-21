use engine::{ComponentStorage, EngineResult, EntityID, ProgressModel, Query};

use crate::resource::*;

use super::{AnimationData, CharacterState, NpcTag};

pub fn npc_update(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query: Query = Query::new().with_component::<NpcTag>();
    for entity_id in storage.fetch_entities(&query) {
        update_character(storage, entity_id)?;
    }
    Ok(())
}

fn update_character(storage: &mut ComponentStorage, entity_id: EntityID) -> EngineResult<()> {
    let Some(state) = storage.get::<CharacterState>(entity_id).map(|x| *x) else {
        return Err(engine::EngineError::ComponentNotFound(
            "NpcState".to_string(),
        ));
    };
    use CharacterState::*;
    match state {
        Idle(mut progress) => {
            if !progress.is_performing() {
                let data = AnimationData::new(NPC_SOLDIER_IDLE);
                storage.set(entity_id, Some(data));
            }
            progress.teak();
            storage.set(entity_id, Some(Damage(progress)));
        }
        Death(mut progress) => {
            if progress.is_completed() {
                storage.remove_entity(entity_id);
            } else {
                if !progress.is_performing() {
                    let data = AnimationData::new(NPC_SOLDIER_DEATH);
                    storage.set(entity_id, Some(data));
                }
                progress.teak();
                storage.set(entity_id, Some(Death(progress)));
            }
        }
        Attack(mut progress) => {
            // NPC_SOLDIER_ATTACK
        }
        Walk(mut progress) => {
            // NPC_SOLDIER_WALK
        }
        Damage(mut progress) => {
            if progress.is_completed() {
                storage.set(entity_id, Some(Idle(ProgressModel::infinite())));
            } else {
                if !progress.is_performing() {
                    let data = AnimationData::new(NPC_SOLDIER_DAMAGE);
                    storage.set(entity_id, Some(data));
                }
                progress.teak();
                storage.set(entity_id, Some(Damage(progress)));
            }
        }
    }
    Ok(())
}
