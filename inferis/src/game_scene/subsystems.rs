use std::borrow::BorrowMut;

use engine::{
    ray_cast, ComponentStorage, EngineError, EngineResult, EntityID, Float, Query, Vec2f,
};

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

pub fn ray_cast_from_entity(
    entity_id: EntityID,
    storage: &ComponentStorage,
    maze_id: EntityID,
    position: Vec2f,
    angle: Float,
) -> EngineResult<Option<EntityID>> {
    let query = Query::new().with_component::<components::BoundingBox>();
    let entities = storage.fetch_entities(&query);
    if entities.is_empty() {
        return Ok(None);
    }
    let check = |point: Vec2f| {
        if point.x < 0.0 || point.y < 0.0 {
            return None;
        }
        let (x, y) = (point.x.round() as i32, point.y.round() as i32);
        for target_id in &entities {
            if *target_id == entity_id {
                continue;
            }
            let Some(pos) = storage.get::<components::Position>(*target_id).map(|x| x.0) else {
                continue;
            };
            let (tx, ty) = (pos.x.round() as i32, pos.y.round() as i32);
            let dist = (pos - point).hypotenuse();
            if x == tx && y == ty || dist < 0.3 {
                return Some(*target_id);
            }
        }
        // --- TEMPORARY
        if let Some(true) = storage
            .get::<components::Maze>(maze_id)
            .map(|x| x.is_wall(point))
        {
            return Some(maze_id);
        };
        // ---
        None
    };
    let result = ray_cast(position, angle, &check);
    Ok(result.value)
}
