use std::{borrow::BorrowMut, f32::consts::PI};

use engine::{ComponentStorage, EngineResult, EntityID, Query, Vec2f};

use super::{Angle, Maze, Position, Transform};

pub fn transform_entities(storage: &mut ComponentStorage) -> EngineResult<()> {
    let query = Query::new().with_component::<Transform>();
    let entities = storage.fetch_entities(&query);
    for entity_id in entities {
        transform_entity(storage, entity_id)?;
        storage.set::<Transform>(entity_id, None);
    }
    Ok(())
}

fn transform_entity(storage: &mut ComponentStorage, entity_id: EntityID) -> EngineResult<()> {
    let Some(transform) = storage.get::<Transform>(entity_id).map(|x| *x) else {
        unreachable!("[transform] query executed successfully but transform component not found")
    };

    if let Some(mut position) = storage.get_mut::<Position>(entity_id) {
        let mut x = position.0.x;
        let mut y = position.0.y;
        if check_collisions(storage, Vec2f::new(x + transform.relative_x, y)) {
            x += transform.relative_x;
        }
        if check_collisions(storage, Vec2f::new(x, y + transform.relative_y)) {
            y += transform.relative_y;
        }
        let pos = position.borrow_mut();
        pos.0 = Vec2f::new(x, y);
    }

    if let Some(mut angle_comp) = storage.get_mut::<Angle>(entity_id) {
        let angle = angle_comp.borrow_mut();
        let mut val = (angle.0 + transform.relative_angle) % (2.0 * PI);
        if val < 0.0 {
            val += 2.0 * PI;
        }
        angle.0 = val;
    }

    Ok(())
}

// trivial collision check
fn check_collisions(storage: &ComponentStorage, position: Vec2f) -> bool {
    // TODO: implement these steps for collider:
    // 1) add bounding box for all objects that are obstacles
    // 2) get list of objects with bounding boxes, take into account id of transformable object to avoid check with itself
    // 3) check box collisions
    {
        // TEMPORARY: now just check the wall collisions
        let query = Query::new().with_component::<Maze>();
        let maze_id = *storage.fetch_entities(&query).get(0).unwrap();
        let Some(maze) = storage.get::<Maze>(maze_id) else {
            panic!("[check_collisions] maze not found")
        };
        !maze.is_wall(position)
    }
}
