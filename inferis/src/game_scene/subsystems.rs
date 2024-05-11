use std::borrow::BorrowMut;

use engine::{ComponentStorage, EngineResult, EntityID};

use crate::{game_scene::components, gameplay::WeaponState};

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

///
pub fn process_damages(
    storage: &ComponentStorage,
    entity_id: EntityID,
    sound_asset_id: impl Into<String>,
) -> EngineResult<()> {
    Ok(())
}
