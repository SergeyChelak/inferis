use std::borrow::BorrowMut;

use engine::{ComponentStorage, EngineResult, EntityID, FrameDuration, Query, Rectangle, Vec2f};

use super::{
    ray_caster::ray_cast, BoundingBox, Damage, Health, Maze, Position, Recharge, RechargeTime, Shot,
};

pub fn attack_system(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new().with_component::<Shot>();
    let entities = storage.fetch_entities(&query);
    for id in entities {
        if is_recharged(storage, id) {
            process_shot(storage, id)?;
        } else {
            println!("[attack] shot discarded due recharge");
        }
        storage.set::<Shot>(id, None);
    }
    Ok(())
}

fn is_recharged(storage: &mut ComponentStorage, entity_id: EntityID) -> bool {
    let is_ready = if let Some(mut comp) = storage.get_mut::<Recharge>(entity_id) {
        comp.0.teak();
        comp.0.is_completed()
    } else {
        true
    };
    if is_ready {
        storage.set::<Recharge>(entity_id, None);
    }
    is_ready
}

fn process_shot(storage: &mut ComponentStorage, entity_id: EntityID) -> EngineResult<()> {
    let Some(damage) = storage.get::<Damage>(entity_id).map(|x| x.0) else {
        println!(
            "[attack] skipped shot for entity {} without damage component",
            entity_id.index()
        );
        return Ok(());
    };
    if let Some(duration) = storage.get::<RechargeTime>(entity_id).map(|x| x.0) {
        let value = Recharge(FrameDuration::new(duration));
        storage.set(entity_id, Some(value));
    }
    let Ok(Some(target_id)) = ray_cast_shot(storage, entity_id) else {
        println!("[attack] no target reached");
        return Ok(());
    };
    if let Some(mut comp) = storage.get_mut::<Health>(target_id) {
        let health = comp.borrow_mut();
        health.0 = health.0.saturating_sub(damage);
    }
    Ok(())
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
    let result = ray_cast(shot.from, shot.angle, &check);
    Ok(result.value)
}
