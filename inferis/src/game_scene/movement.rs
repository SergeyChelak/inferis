use std::f32::consts::PI;

use engine::{
    refresh_cached_entity,
    systems::{GameSystem, GameSystemCommand},
    ComponentStorage, EngineError, EngineResult, EntityID, Query, Rectangle, Vec2f,
};

use super::components;

#[derive(Default)]
pub struct MovementSystem {
    maze_id: EntityID,
}

impl MovementSystem {
    pub fn new() -> Self {
        Default::default()
    }

    fn update_storage_cache(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        refresh_cached_entity::<components::Maze>(storage, &mut self.maze_id, "[v2.movement] maze")
    }

    fn move_entity(&self, storage: &mut ComponentStorage, entity_id: EntityID) -> EngineResult<()> {
        let Some(movement) = storage.get::<components::Movement>(entity_id).map(|x| *x) else {
            return Err(EngineError::unexpected_state(
                "[v2.movement] query executed successfully but transform component not found",
            ));
        };
        if let Some(mut position) = storage.get_mut::<components::Position>(entity_id) {
            let mut x = position.0.x;
            let mut y = position.0.y;
            // entities without a bounding box don't move
            if let Some(size) = storage
                .get::<components::BoundingBox>(entity_id)
                .map(|b| b.0)
            {
                // gather the collision context once instead of on every axis check
                let obstacles = obstacle_rects(storage, entity_id);
                let Some(maze) = storage.get::<components::Maze>(self.maze_id) else {
                    return Err(EngineError::unexpected_state(
                        "[v2.movement] maze component not found",
                    ));
                };
                let can_move = |target: Vec2f| {
                    let rect = Rectangle::with_pole(target, size);
                    obstacles.iter().all(|other| !rect.has_intersection(other))
                        && !maze.is_wall(target)
                };
                if can_move(Vec2f::new(x + movement.x, y)) {
                    x += movement.x;
                }
                if can_move(Vec2f::new(x, y + movement.y)) {
                    y += movement.y;
                }
            }
            position.0 = Vec2f::new(x, y);
        }

        if let Some(mut angle_comp) = storage.get_mut::<components::Angle>(entity_id) {
            let mut val = (angle_comp.0 + movement.angle) % (2.0 * PI);
            if val < 0.0 {
                val += 2.0 * PI;
            }
            angle_comp.0 = val;
        }
        storage.set::<components::Movement>(entity_id, None);
        Ok(())
    }
}

// TODO: implement these steps for collider:
// 1) add bounding box for all objects that are obstacles
// 2) get list of objects with bounding boxes, take into account id of transformable object to avoid check with itself
// 3) check box collisions
fn obstacle_rects(storage: &ComponentStorage, entity_id: EntityID) -> Vec<Rectangle> {
    let query = Query::new()
        .with_component::<components::BoundingBox>()
        .with_component::<components::Position>();
    storage
        .fetch_entities(&query)
        .into_iter()
        .filter(|id| *id != entity_id)
        .filter_map(|id| {
            let size = storage.get::<components::BoundingBox>(id).map(|x| x.0)?;
            let position = storage.get::<components::Position>(id).map(|x| x.0)?;
            Some(Rectangle::with_pole(position, size))
        })
        .collect()
}

impl GameSystem for MovementSystem {
    fn setup(
        &mut self,
        storage: &mut ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> EngineResult<()> {
        self.update_storage_cache(storage)?;
        println!("[v2.movement] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        _frames: usize,
        _delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<engine::systems::GameSystemCommand> {
        self.update_storage_cache(storage)?;

        let query = Query::new().with_component::<components::Movement>();
        let entities = storage.fetch_entities(&query);
        for entity_id in entities {
            self.move_entity(storage, entity_id)?;
        }
        Ok(GameSystemCommand::Nothing)
    }
}
