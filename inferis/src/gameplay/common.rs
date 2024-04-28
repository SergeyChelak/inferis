use engine::{ray_cast, ComponentStorage, EngineResult, EntityID, Float, Query, Vec2f};

use super::{BoundingBox, Maze, Position};

pub fn ray_cast_with_entity(
    storage: &mut ComponentStorage,
    entity_id: EntityID,
    position: Vec2f,
    angle: Float,
) -> EngineResult<Option<EntityID>> {
    let query = Query::new().with_component::<BoundingBox>();
    let entities = storage.fetch_entities(&query);
    if entities.is_empty() {
        return Ok(None);
    }
    // --- TEMPORARY
    let query = Query::new().with_component::<Maze>();
    let maze_id = *storage.fetch_entities(&query).first().unwrap();
    // ---
    let check = |point: Vec2f| {
        if point.x < 0.0 || point.y < 0.0 {
            return None;
        }
        let (x, y) = (point.x.round() as i32, point.y.round() as i32);
        for target_id in &entities {
            if *target_id == entity_id {
                continue;
            }
            let Some(pos) = storage.get::<Position>(*target_id).map(|x| x.0) else {
                continue;
            };
            let (tx, ty) = (pos.x.round() as i32, pos.y.round() as i32);
            let dist = (pos - point).hypotenuse();
            if x == tx && y == ty || dist < 0.3 {
                return Some(*target_id);
            }
        }
        // --- TEMPORARY
        if let Some(true) = storage.get::<Maze>(maze_id).map(|x| x.is_wall(point)) {
            return Some(maze_id);
        };
        // ---
        None
    };
    let result = ray_cast(position, angle, &check);
    Ok(result.value)
}
