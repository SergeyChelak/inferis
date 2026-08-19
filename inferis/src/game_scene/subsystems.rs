use engine::{ray_cast, ComponentStorage, EngineResult, EntityID, Float, Query, Rectangle, Vec2f};

use crate::game_scene::components;

use super::components::{ActorState, BoundingBox};

/// Updates weapon state to new one if it reached frame deadline
/// if state doesn't changed functions returns None
pub fn update_weapon_state(
    frame: usize,
    storage: &mut ComponentStorage,
    entity_id: EntityID,
) -> Option<components::WeaponState> {
    let has_shot = storage
        .get::<components::Shot>(entity_id)
        .map(|x| x.deadline > frame)
        .unwrap_or(false);
    let mut weapon = storage.get_mut::<components::Weapon>(entity_id)?;
    use components::WeaponState::*;
    let new_state = match weapon.state {
        Undefined => Ready(usize::MAX),
        Recharge(deadline) if deadline <= frame => Ready(usize::MAX),
        Ready(_) if has_shot => Recharge(frame + weapon.recharge_time),
        state => state,
    };
    if new_state != weapon.state {
        if let (Ready(_), Recharge(_)) = (weapon.state, new_state) {
            weapon.ammo_count = weapon.ammo_count.saturating_sub(1);
        }
        weapon.state = new_state;
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
    storage.set::<components::Damage>(entity_id, None)?;
    let health = {
        let Some(mut comp) = storage.get_mut::<components::Health>(entity_id) else {
            return Err(engine::EngineError::component_not_found(
                "[actor state] Health",
            ));
        };
        comp.0 = comp.0.saturating_sub(damage);
        comp.0
    };
    let state = if health > 0 {
        components::ActorState::Damaged(damage_deadline)
    } else {
        components::ActorState::Dead(usize::MAX)
    };
    storage.set(entity_id, Some(state))?;
    Ok(Some(state))
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
        Damaged(deadline) if deadline <= frame => Some(Idle(usize::MAX)),
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
    matches!(weapon.state, components::WeaponState::Ready(_)) && weapon.ammo_count > 0
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

    // borrow the maze once: fetching the component inside the closure
    // costs a map lookup, RefCell borrow and downcast on every ray step
    let maze = storage.get::<components::Maze>(maze_id);
    let check_wall = |point: Vec2f| {
        if point.x < 0.0 || point.y < 0.0 {
            return None;
        }
        match &maze {
            Some(maze) if maze.is_wall(point) => Some(maze_id),
            _ => None,
        }
    };

    // no maze means no walls to hit, so there is nothing to step through
    let max_steps = maze
        .as_ref()
        .map(|maze| maze.ray_cast_steps())
        .unwrap_or_default();
    let ray_result = ray_cast(position, angle, max_steps, &check_wall);
    let mut min_dist = if ray_result.value.is_some() {
        ray_result.depth
    } else {
        Float::INFINITY
    };
    let mut closest_entity = None;

    if !entities.is_empty() {
        let ray_dir = Vec2f::new(angle.cos(), angle.sin());
        for target_id in entities {
            if target_id == entity_id {
                continue;
            }
            let Some(pos) = storage.get::<components::Position>(target_id).map(|x| x.0) else {
                continue;
            };
            let Some(target_size) = storage.get::<BoundingBox>(target_id).map(|x| x.0) else {
                continue;
            };
            let rect = Rectangle::with_pole(pos, target_size);
            if let Some(t) = rect.ray_intersect(position, ray_dir) {
                if t >= 0.0 && t < min_dist {
                    min_dist = t;
                    closest_entity = Some(target_id);
                }
            }
        }
    }

    if let Some(target) = closest_entity {
        Ok(Some(target))
    } else if ray_result.value.is_some() {
        Ok(Some(maze_id))
    } else {
        Ok(None)
    }
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

/// Updates ActorState
/// Returns new state value if its enum is different
pub fn replace_actor_state(
    state: ActorState,
    storage: &mut ComponentStorage,
    entity_id: EntityID,
) -> EngineResult<Option<ActorState>> {
    let current = get_actor_state(storage, entity_id);
    use std::mem;
    if mem::discriminant(&current) == mem::discriminant(&state) {
        return Ok(None);
    }
    storage.set(entity_id, Some(state))?;
    Ok(Some(state))
}
