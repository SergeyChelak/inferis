use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
};

use engine::{
    frame_counter::{FrameCounterService, FrameCounterState},
    ComponentStorage, EngineError, EngineResult, EntityID, Query,
};

use super::*;

pub fn state_system(
    storage: &mut ComponentStorage,
    frame_counter: &mut FrameCounterService,
) -> EngineResult<()> {
    process_damages(storage)?;
    update_player(storage)?;
    update_all_npc(storage, frame_counter)?;
    cleanup(storage)?;
    Ok(())
}

fn update_all_npc(
    storage: &mut ComponentStorage,
    frame_counter: &mut FrameCounterService,
) -> EngineResult<()> {
    let query = Query::new().with_component::<NpcTag>();
    let entities = storage.fetch_entities(&query);
    for entity_id in entities {
        update_npc(storage, frame_counter, entity_id)?;
    }
    Ok(())
}

fn update_npc(
    storage: &mut ComponentStorage,
    frame_counter: &mut FrameCounterService,
    entity_id: EntityID,
) -> EngineResult<()> {
    let key = npc_state_frame_counter_key(entity_id);
    use CharacterState::*;
    if storage.has_component::<ReceivedDamage>(entity_id) {
        storage.set::<CharacterState>(entity_id, Some(Damage));
        frame_counter.add_counter(key.clone(), 15);
    }
    let Some(state) = storage.get::<CharacterState>(entity_id).map(|x| *x) else {
        return Err(EngineError::component_not_found("CharacterState"));
    };
    let is_completed = frame_counter
        .state(&key)
        .map(|x| matches!(x, FrameCounterState::Completed))
        .unwrap_or(false);
    if is_completed {
        frame_counter.remove(&key);
    }
    match (state, is_completed) {
        (Damage, true) => {
            if let Some(true) = storage.get::<Health>(entity_id).map(|x| x.0 > 0) {
                storage.set(entity_id, Some(Idle));
            } else {
                storage.set(entity_id, Some(Death));
                frame_counter.add_counter(key.clone(), 30);
            }
        }
        (Death, true) => {
            storage.remove_entity(entity_id);
        }
        _ => {
            // don't change state
        }
    }
    update_npc_animation(storage, entity_id)?;
    Ok(())
}

fn update_npc_animation(storage: &mut ComponentStorage, entity_id: EntityID) -> EngineResult<()> {
    let Some(state) = storage.get::<CharacterState>(entity_id).map(|x| *x) else {
        // entity can be removed
        return Ok(());
    };
    use CharacterState::*;
    let animation_id = match state {
        Idle => NPC_SOLDIER_IDLE,
        Damage => NPC_SOLDIER_DAMAGE,
        Walk => NPC_SOLDIER_WALK,
        Death => NPC_SOLDIER_DEATH,
        Attack => NPC_SOLDIER_ATTACK,
    };
    update_or_insert_animation(storage, entity_id, animation_id);
    Ok(())
}

fn process_damages(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new()
        .with_component::<ReceivedDamage>()
        .with_component::<Health>();
    let entities = storage.fetch_entities(&query);
    for entity_id in entities {
        let Some(damage) = storage.get::<ReceivedDamage>(entity_id).map(|x| x.0) else {
            return Err(EngineError::component_not_found("ReceivedDamage"));
        };
        let Some(mut comp) = storage.get_mut::<Health>(entity_id) else {
            return Err(EngineError::component_not_found("Health"));
        };
        let health = comp.borrow_mut();
        health.0 = health.0.saturating_sub(damage);
        println!("[state] entity {} health {}", entity_id.id_key(), health.0);
    }
    Ok(())
}

fn update_player(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new().with_component::<PlayerTag>();
    let Some(&player_id) = storage.fetch_entities(&query).first() else {
        return Err(EngineError::unexpected_state(
            "Failed to query entity id with player tag",
        ));
    };
    // TODO: check player is alive
    update_player_weapon(storage, player_id)?;
    Ok(())
}

fn update_player_weapon(storage: &mut ComponentStorage, player_id: EntityID) -> EngineResult<()> {
    let Some(weapon) = storage.get::<Weapon>(player_id).map(|x| *x) else {
        return Err(EngineError::component_not_found("Weapon"));
    };
    let animation_id = if matches!(weapon.state, super::WeaponState::Ready) {
        PLAYER_SHOTGUN_IDLE_ANIM
    } else {
        PLAYER_SHOTGUN_SHOT_ANIM
    };
    update_or_insert_animation(storage, player_id, animation_id);
    Ok(())
}

fn cleanup(storage: &mut ComponentStorage) -> EngineResult<()> {
    cleanup_component::<Shot>(storage)?;
    cleanup_component::<ReceivedDamage>(storage)?;
    Ok(())
}

// utils
fn cleanup_component<T: Any>(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new().with_component::<T>();
    let entities = storage.fetch_entities(&query);
    for id in entities {
        if !storage.set::<T>(id, None) {
            println!("[state] failed to remove component {:?}", TypeId::of::<T>());
        }
    }
    Ok(())
}

fn update_or_insert_animation(
    storage: &mut ComponentStorage,
    entity_id: EntityID,
    animation_id: &str,
) {
    if storage.has_component::<AnimationData>(entity_id) {
        if let Some(mut comp) = storage.get_mut::<AnimationData>(entity_id) {
            let anim = comp.borrow_mut();
            anim.replace(animation_id);
        }
    } else {
        let data = AnimationData::new(animation_id);
        storage.set(entity_id, Some(data));
    }
}

fn npc_state_frame_counter_key(entity_id: EntityID) -> String {
    format!("NPC_STATE_{}", entity_id.id_key())
}
