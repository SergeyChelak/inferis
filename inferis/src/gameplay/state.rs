use std::{any::Any, borrow::BorrowMut};

use engine::{ComponentStorage, EngineError, EngineResult, EntityID, Query};

use super::*;

pub fn state_system(storage: &mut ComponentStorage) -> EngineResult<()> {
    process_damages(storage)?;
    update_player(storage)?;
    cleanup(storage)?;
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
        println!("entity {} health {}", entity_id.id_key(), health.0);
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
    if storage.get::<AnimationData>(player_id).is_none() {
        let data = AnimationData::new(animation_id);
        storage.set(player_id, Some(data));
    }
    if let Some(mut comp) = storage.get_mut::<AnimationData>(player_id) {
        let anim = comp.borrow_mut();
        anim.replace(animation_id);
    }
    Ok(())
}

fn cleanup(storage: &mut ComponentStorage) -> EngineResult<()> {
    cleanup_component::<Shot>(storage)?;
    cleanup_component::<ReceivedDamage>(storage)?;
    Ok(())
}

fn cleanup_component<T: Any>(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new().with_component::<T>();
    let entities = storage.fetch_entities(&query);
    for id in entities {
        storage.set::<T>(id, None);
    }
    Ok(())
}
