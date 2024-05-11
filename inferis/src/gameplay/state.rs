use std::borrow::BorrowMut;

use engine::{
    cleanup_component, frame_counter::AggregatedFrameCounter, ComponentStorage, EngineError,
    EngineResult, EntityID, Query,
};

use self::generator::LevelData;

use super::*;

pub fn state_system(
    storage: &mut ComponentStorage,
    frame_counter: &mut AggregatedFrameCounter,
    level_data: &LevelData,
) -> EngineResult<()> {
    process_damages(storage)?;
    update_player(storage, level_data)?;
    update_all_npc(storage, frame_counter)?;
    Ok(())
}

fn update_all_npc(
    storage: &mut ComponentStorage,
    frame_counter: &mut AggregatedFrameCounter,
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
    frame_counter: &mut AggregatedFrameCounter,
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
    let is_completed = frame_counter.is_completed(&key);
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

                let sound_fx = SoundFx::once(SOUND_NPC_DEATH);
                storage.set(entity_id, Some(sound_fx));
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
        add_damage_sound(storage, entity_id)?;
        let Some(mut comp) = storage.get_mut::<Health>(entity_id) else {
            return Err(EngineError::component_not_found("Health"));
        };
        let health = comp.borrow_mut();
        health.0 = health.0.saturating_sub(damage);
    }
    Ok(())
}

fn add_damage_sound(storage: &mut ComponentStorage, entity_id: EntityID) -> EngineResult<()> {
    let id = if storage.has_component::<PlayerTag>(entity_id) {
        SOUND_PLAYER_PAIN
    } else {
        SOUND_NPC_PAIN
    };
    let sound_fx = SoundFx::once(id);
    storage.set(entity_id, Some(sound_fx));
    Ok(())
}

fn update_player(storage: &mut ComponentStorage, level_data: &LevelData) -> EngineResult<()> {
    let player_id = level_data.player_id;
    let Some(health) = storage.get::<Health>(player_id).map(|x| x.0) else {
        return Ok(());
    };
    if health == 0 {
        storage.set::<UserControllableTag>(player_id, None);
        storage.set::<BoundingBox>(player_id, None);
        storage.set::<Health>(player_id, None);
        storage.set::<AnimationData>(player_id, None);
        return Ok(());
    }
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

pub fn cleanup_system(storage: &mut ComponentStorage) -> EngineResult<()> {
    cleanup_component::<Shot>(storage)?;
    cleanup_component::<ReceivedDamage>(storage)?;
    cleanup_component::<Movement>(storage)?;
    cleanup_component::<SoundFx>(storage)?;
    Ok(())
}

// utils

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
