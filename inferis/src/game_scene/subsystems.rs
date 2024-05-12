use std::borrow::BorrowMut;

use engine::{ComponentStorage, EngineError, EngineResult, EntityID};

use crate::{game_scene::components, gameplay::WeaponState};

use super::components::{ActorState, Damage};

/// Updates weapon state to new one if it reached frame deadline
/// if state doesn't changed functions returns None
pub fn update_weapon_state(
    frame: usize,
    storage: &mut ComponentStorage,
    entity_id: EntityID,
) -> Option<WeaponState> {
    let has_shot = storage
        .get::<components::Shot>(entity_id)
        .map(|x| x.deadline > frame)
        .unwrap_or(false);
    let Some(mut weapon) = storage.get_mut::<components::Weapon>(entity_id) else {
        return None;
    };
    use components::WeaponState::*;
    let new_state = match weapon.state {
        Undefined => Ready(usize::MAX),
        Recharge(deadline) if deadline <= frame => Ready(usize::MAX),
        Ready(_) if has_shot => Recharge(frame + weapon.recharge_time),
        state => state,
    };
    if new_state != weapon.state {
        weapon.borrow_mut().state = new_state;
        Some(new_state)
    } else {
        None
    }
}

/// Checks if weapon is ready for shooting
/// returns false if the Weapon component is missing
pub fn can_shoot(storage: &ComponentStorage, entity_id: EntityID) -> bool {
    let Some(weapon) = storage.get::<components::Weapon>(entity_id) else {
        return false;
    };
    matches!(weapon.state, WeaponState::Ready(_))
}

/// Checks if actor did receive damage. In that case
/// - the 'Damage' component will removed for entity
/// - actor state updated to 'Damaged' with value of 'deadline'
/// - added entity-specific sound
/// - decreased actor health
/// Missing health component threated as an error
pub fn process_damages(
    storage: &mut ComponentStorage,
    entity_id: EntityID,
    deadline: usize,
    sound_asset_id: impl Into<String>,
) -> EngineResult<()> {
    let Some(damage) = storage.get::<Damage>(entity_id).map(|x| x.0) else {
        return Ok(());
    };
    storage.set::<Damage>(entity_id, None);
    if deadline > 0 {
        storage.set(entity_id, Some(ActorState::Damaged(deadline)));
    }
    let sound_fx = components::SoundFx::once(sound_asset_id);
    storage.set(entity_id, Some(sound_fx));

    let Some(mut comp) = storage.get_mut::<components::Health>(entity_id) else {
        return Err(EngineError::component_not_found("[process_damages] Health"));
    };
    let health = comp.borrow_mut();
    health.0 = health.0.saturating_sub(damage);
    Ok(())
}
