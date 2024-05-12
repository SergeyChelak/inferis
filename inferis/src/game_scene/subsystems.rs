use std::borrow::BorrowMut;

use engine::{
    fetch_first, ray_cast, ComponentStorage, EngineResult, EntityID, Float, Query, Vec2f,
};

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

pub fn updated_state(
    frame: usize,
    storage: &mut engine::ComponentStorage,
    entity_id: EntityID,
    damage_duration: usize,
) -> EngineResult<Option<components::ActorState>> {
    let mut state = state_if_damaged(storage, entity_id, frame + damage_duration)?;
    if state.is_none() {
        state = updated_state_if_time(frame, storage, entity_id)?;
    }
    Ok(state)
}

fn state_if_damaged(
    storage: &mut engine::ComponentStorage,
    entity_id: EntityID,
    damage_deadline: usize,
) -> EngineResult<Option<components::ActorState>> {
    let Some(damage) = storage.get::<components::Damage>(entity_id).map(|x| x.0) else {
        return Ok(None);
    };
    storage.set::<components::Damage>(entity_id, None);
    let health = {
        let Some(mut comp) = storage.get_mut::<components::Health>(entity_id) else {
            return Err(engine::EngineError::component_not_found(
                "[actor state] Health",
            ));
        };
        let health = comp.borrow_mut();
        health.0 = health.0.saturating_sub(damage);
        health.0
    };
    let state = if health > 0 {
        components::ActorState::Damaged(damage_deadline)
    } else {
        components::ActorState::Dead(usize::MAX)
    };
    storage.set(entity_id, Some(state));
    return Ok(Some(state));
}

fn updated_state_if_time(
    frame: usize,
    storage: &mut engine::ComponentStorage,
    entity_id: EntityID,
) -> EngineResult<Option<components::ActorState>> {
    let Some(state) = storage.get::<components::ActorState>(entity_id).map(|x| *x) else {
        return Err(engine::EngineError::component_not_found(
            "[actor state] ActorState",
        ));
    };
    use components::ActorState::*;
    let result = match state {
        Undefined => Some(Idle(usize::MAX)),
        Damaged(deadline) if deadline == frame => Some(Idle(usize::MAX)),
        _ => None,
    };
    Ok(result)
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

pub fn fetch_player_id(storage: &ComponentStorage) -> Option<EntityID> {
    fetch_first::<components::PlayerTag>(storage)
}

pub fn get_actor_state(storage: &ComponentStorage, entity_id: EntityID) -> components::ActorState {
    storage
        .get::<components::ActorState>(entity_id)
        .map(|x| *x)
        .unwrap_or_default()
}

pub fn is_actor_dead(storage: &ComponentStorage, entity_id: EntityID) -> bool {
    matches!(
        get_actor_state(storage, entity_id),
        components::ActorState::Dead(_)
    )
}
