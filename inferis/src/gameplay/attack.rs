use std::borrow::BorrowMut;

use engine::{
    frame_counter::AggregatedFrameCounter, ComponentStorage, EngineError, EngineResult, EntityID,
    Query,
};

use super::{
    common::ray_cast_with_entity, PlayerTag, ReceivedDamage, Shot, ShotState, SoundFx, Weapon,
    WeaponState, SOUND_NPC_ATTACK, SOUND_PLAYER_ATTACK,
};

pub fn attack_system(
    storage: &mut ComponentStorage,
    frame_counter: &mut AggregatedFrameCounter,
) -> EngineResult<()> {
    process_shorts(storage, frame_counter)?;
    refresh_weapon_state(storage, frame_counter)?;
    Ok(())
}

fn refresh_weapon_state(
    storage: &mut ComponentStorage,
    frame_counter: &mut AggregatedFrameCounter,
) -> EngineResult<()> {
    let query = Query::new().with_component::<Weapon>();
    let entities = storage.fetch_entities(&query);
    for entity_id in entities {
        if !frame_counter.is_completed(&frame_counter_key(entity_id)) {
            continue;
        };
        let Some(mut comp) = storage.get_mut::<Weapon>(entity_id) else {
            continue;
        };
        let weapon = comp.borrow_mut();
        weapon.state = WeaponState::Ready(usize::MAX);
    }
    Ok(())
}

fn frame_counter_key(entity_id: EntityID) -> String {
    format!("WEAPON_{}", entity_id.id_key())
}

fn process_shorts(
    storage: &mut ComponentStorage,
    frame_counter: &mut AggregatedFrameCounter,
) -> EngineResult<()> {
    let query = Query::new().with_component::<Shot>();
    let entities = storage.fetch_entities(&query);
    for entity_id in entities {
        let Some(true) = storage
            .get::<Shot>(entity_id)
            .map(|x| matches!(x.state, ShotState::Initial))
        else {
            continue;
        };
        let new_state = if try_shot(storage, frame_counter, entity_id)? {
            ShotState::Accepted
        } else {
            ShotState::Cancelled
        };
        let Some(mut shot) = storage.get_mut::<Shot>(entity_id) else {
            return Err(EngineError::component_not_found("Shot"));
        };
        shot.borrow_mut().state = ShotState::Cancelled;
    }
    Ok(())
}

fn try_shot(
    storage: &mut ComponentStorage,
    frame_counter: &mut AggregatedFrameCounter,
    entity_id: EntityID,
) -> EngineResult<bool> {
    let Some(weapon) = storage.get::<Weapon>(entity_id).map(|x| *x) else {
        return Err(EngineError::component_not_found("Weapon"));
    };
    if weapon.ammo_count == 0 || matches!(weapon.state, WeaponState::Recharge(_)) {
        return Ok(false);
    }
    if let Ok(Some(target_id)) = ray_cast_shot(storage, entity_id) {
        let total_damage = weapon.damage
            + storage
                .get::<ReceivedDamage>(target_id)
                .map(|x| x.0)
                .unwrap_or_default();
        storage.set::<ReceivedDamage>(target_id, Some(ReceivedDamage(total_damage)));
    }
    if let Some(mut comp) = storage.get_mut::<Weapon>(entity_id) {
        let w = comp.borrow_mut();
        w.ammo_count = weapon.ammo_count.saturating_sub(1);
        w.state = WeaponState::Recharge(0);
        frame_counter.add_counter(frame_counter_key(entity_id), weapon.recharge_time);
    };
    add_shoot_sound(storage, entity_id)?;
    Ok(true)
}

fn ray_cast_shot(
    storage: &mut ComponentStorage,
    entity_id: EntityID,
) -> EngineResult<Option<EntityID>> {
    let Some(shot) = storage.get::<Shot>(entity_id).map(|x| *x) else {
        return Err(EngineError::component_not_found("Shot"));
    };
    ray_cast_with_entity(storage, entity_id, shot.position, shot.angle)
}

fn add_shoot_sound(storage: &mut ComponentStorage, entity_id: EntityID) -> EngineResult<()> {
    let id = if storage.has_component::<PlayerTag>(entity_id) {
        SOUND_PLAYER_ATTACK
    } else {
        SOUND_NPC_ATTACK
    };
    let sound_fx = SoundFx::once(id);
    storage.set(entity_id, Some(sound_fx));
    Ok(())
}
