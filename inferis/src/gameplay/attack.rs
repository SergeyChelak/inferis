use std::borrow::BorrowMut;

use engine::{ComponentStorage, EngineError, EngineResult, EntityID, Query, Rectangle, Vec2f};

use super::{
    ray_caster::ray_cast, BoundingBox, Health, Maze, Position, ReceivedDamage, Shot, ShotState,
    Weapon, WeaponState,
};

pub fn attack_system(storage: &mut ComponentStorage) -> EngineResult<()> {
    process_shorts(storage)?;
    refresh_weapon_state(storage)?;
    Ok(())
}

fn refresh_weapon_state(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new().with_component::<Weapon>();
    let entities = storage.fetch_entities(&query);
    for entity_id in entities {
        // TODO: check if weapon recharge time is out
    }
    Ok(())
}

fn process_shorts(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new().with_component::<Shot>();
    let entities = storage.fetch_entities(&query);
    for entity_id in entities {
        let Some(true) = storage
            .get::<Shot>(entity_id)
            .map(|x| matches!(x.state, ShotState::Initial))
        else {
            continue;
        };
        let new_state = if try_shot(storage, entity_id)? {
            ShotState::Accepted
        } else {
            ShotState::Cancelled
        };
        let Some(mut shot) = storage.get_mut::<Shot>(entity_id) else {
            return Err(EngineError::component_not_found("Shot"));
        };
        shot.borrow_mut().state = new_state;
    }
    Ok(())
}

fn try_shot(storage: &mut ComponentStorage, entity_id: EntityID) -> EngineResult<bool> {
    let Some(weapon) = storage.get::<Weapon>(entity_id).map(|x| *x) else {
        return Err(EngineError::component_not_found("Weapon"));
    };
    if weapon.ammo_count == 0 || matches!(weapon.state, WeaponState::Recharge) {
        println!("[attack] shot discarded due recharge or empty clip");
        return Ok(false);
    }
    let Ok(Some(target_id)) = ray_cast_shot(storage, entity_id) else {
        println!("[attack] no target reached");
        return Ok(true);
    };
    let total_damage = weapon.damage
        + storage
            .get::<ReceivedDamage>(target_id)
            .map(|x| x.0)
            .unwrap_or_default();
    storage.set::<ReceivedDamage>(entity_id, Some(ReceivedDamage(total_damage)));
    if let Some(mut comp) = storage.get_mut::<Weapon>(entity_id) {
        let w = comp.borrow_mut();
        w.ammo_count = weapon.ammo_count.saturating_sub(1);
        w.state = WeaponState::Recharge;
    };
    Ok(true)
}

fn ray_cast_shot(
    storage: &mut ComponentStorage,
    entity_id: EntityID,
) -> EngineResult<Option<EntityID>> {
    let Some(shot) = storage.get::<Shot>(entity_id) else {
        unreachable!()
    };
    let query = Query::new()
        .with_component::<Health>()
        .with_component::<BoundingBox>();
    let entities = storage.fetch_entities(&query);
    if entities.is_empty() {
        println!("[attack] no targets to shoot");
        return Ok(None);
    }
    // --- TEMPORARY
    let query = Query::new().with_component::<Maze>();
    let maze_id = *storage.fetch_entities(&query).get(0).unwrap();
    // ---
    let check = |point: Vec2f| {
        for target_id in &entities {
            if *target_id == entity_id {
                continue;
            }
            // --- TEMPORARY
            if let Some(true) = storage.get::<Maze>(maze_id).map(|x| x.is_wall(point)) {
                println!("[attack] shoot in the wall");
                return None;
            };
            // ---

            let Some(pos) = storage.get::<Position>(*target_id).map(|x| x.0) else {
                continue;
            };
            let Some(bounding_box) = storage.get::<BoundingBox>(*target_id).map(|x| x.0) else {
                continue;
            };
            let rect = Rectangle::with_pole(pos, bounding_box);
            if rect.contains(point) {
                println!("[attack] attacked enemy with id {}", target_id.index());
                return Some(*target_id);
            }
        }
        None
    };
    let result = ray_cast(shot.position, shot.angle, &check);
    Ok(result.value)
}
