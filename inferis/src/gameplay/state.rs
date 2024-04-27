use std::borrow::BorrowMut;

use engine::{ComponentStorage, EngineError, EngineResult, Query};

use super::*;

pub fn state_system(storage: &mut ComponentStorage) -> EngineResult<()> {
    update_player_weapon(storage)?;
    cleanup_shots(storage)?;
    Ok(())
}

fn update_player_weapon(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new()
        .with_component::<Weapon>()
        .with_component::<PlayerTag>();
    let Some(&player_id) = storage.fetch_entities(&query).first() else {
        return Err(EngineError::unexpected_state(
            "Query {PlayerTag; Weapon} returned nothing",
        ));
    };
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

fn cleanup_shots(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new().with_component::<Shot>();
    let entities = storage.fetch_entities(&query);
    for id in entities {
        storage.set::<Shot>(id, None);
    }
    Ok(())
}
