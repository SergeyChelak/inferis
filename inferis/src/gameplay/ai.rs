use engine::{ComponentStorage, EngineError, EngineResult, EntityID, Float, Query};

use super::{common::ray_cast_with_entity, CharacterState, NpcTag, Position};

pub fn ai_system(storage: &mut ComponentStorage, player_id: EntityID) -> EngineResult<()> {
    let query = Query::new().with_component::<NpcTag>();
    let entities = storage.fetch_entities(&query);
    for npc_id in entities {
        npc_behavior(storage, npc_id, player_id)?;
    }
    Ok(())
}

fn npc_behavior(
    storage: &mut ComponentStorage,
    npc_id: EntityID,
    player_id: EntityID,
) -> EngineResult<()> {
    let Some(state) = storage.get::<CharacterState>(npc_id).map(|x| *x) else {
        return Err(EngineError::component_not_found("CharacterState"));
    };
    use CharacterState::*;
    if matches!(state, Death | Damage) {
        return Ok(());
    }
    if let Some(distance) = vision(storage, npc_id, player_id) {
        let state = if distance < 10.0 { Attack } else { Walk };
        storage.set(npc_id, Some(state));
    }
    // else {
    //     storage.set(npc_id, Some(Idle));
    // }
    Ok(())
}

fn vision(storage: &mut ComponentStorage, npc_id: EntityID, player_id: EntityID) -> Option<Float> {
    let Some(npc_position) = storage.get_mut::<Position>(npc_id).map(|x| x.0) else {
        return None;
    };
    let Some(player_position) = storage.get_mut::<Position>(player_id).map(|x| x.0) else {
        return None;
    };
    let vector = player_position - npc_position;
    let angle = vector.y.atan2(vector.x);
    let Some(entity_id) = ray_cast_with_entity(storage, npc_id, npc_position, angle)
        .ok()
        .and_then(|x| x)
    else {
        return None;
    };
    if entity_id != player_id {
        return None;
    }
    Some(vector.hypotenuse())
}
