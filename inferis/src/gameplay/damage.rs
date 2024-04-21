use std::borrow::BorrowMut;

use engine::{ComponentStorage, EngineResult, EntityID, ProgressModel};

use super::{CharacterState, Health, HealthType};

pub fn make_damage(
    storage: &mut ComponentStorage,
    character_id: EntityID,
    value: HealthType,
) -> EngineResult<()> {
    let health = {
        let Some(mut health) = storage.get_mut::<Health>(character_id) else {
            return Ok(());
        };
        let h = health.0.saturating_sub(value);
        health.borrow_mut().0 = h;
        h
    };
    if health == 0 {
        storage.set(
            character_id,
            Some(CharacterState::Death(ProgressModel::new(60))),
        );
    } else {
        storage.set(
            character_id,
            Some(CharacterState::Damage(ProgressModel::new(30))),
        );
    }
    Ok(())
}
