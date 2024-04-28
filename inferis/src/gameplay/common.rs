use engine::{ray_cast, ComponentStorage, EngineResult, EntityID, Float, Query, Rectangle, Vec2f};

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
        // println!("[ray_cast] no targets to cast");
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
            let Some(pos) = storage.get::<Position>(*target_id).map(|x| x.0) else {
                continue;
            };
            let Some(bounding_box) = storage.get::<BoundingBox>(*target_id).map(|x| x.0) else {
                continue;
            };
            let rect = Rectangle::with_pole(pos, bounding_box);
            if rect.contains(point) {
                // println!(
                //     "[ray_cast] casted to the enemy with id {}",
                //     target_id.index()
                // );
                return Some(*target_id);
            }
        }
        // --- TEMPORARY
        if let Some(true) = storage.get::<Maze>(maze_id).map(|x| x.is_wall(point)) {
            // println!("[ray_cast] casted into the wall");
            return Some(maze_id);
        };
        // ---
        None
    };
    let result = ray_cast(position, angle, &check);
    Ok(result.value)
}
