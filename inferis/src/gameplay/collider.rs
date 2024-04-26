use std::borrow::BorrowMut;

use engine::{ComponentStorage, EngineError, EngineResult, EntityID};

use super::{Maze, Position, PrevPosition};

pub fn run_collider(
    storage: &mut ComponentStorage,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    check_wall_collisions(storage, player_id, maze_id)?;
    Ok(())
}

fn check_wall_collisions(
    storage: &mut ComponentStorage,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    let Some(maze) = storage.get::<Maze>(maze_id) else {
        return Err(EngineError::component_not_found("Maze"));
    };
    let Some(pos) = storage.get::<Position>(player_id).map(|x| x.0) else {
        return Err(EngineError::component_not_found("Position"));
    };
    let Some(prev_pos) = storage.get::<PrevPosition>(player_id).map(|x| x.0) else {
        return Err(EngineError::component_not_found("PrevPosition"));
    };
    if maze.is_wall(pos) {
        // restore last valid position
        let Some(mut pos) = storage.get_mut::<Position>(player_id) else {
            return Err(EngineError::component_not_found("Position"));
        };
        pos.borrow_mut().0 = prev_pos;
    } else {
        // update with last valid position
        let Some(mut prev_pos) = storage.get_mut::<PrevPosition>(player_id) else {
            return Err(EngineError::component_not_found("PrevPosition"));
        };
        prev_pos.borrow_mut().0 = pos;
    }
    Ok(())
}
