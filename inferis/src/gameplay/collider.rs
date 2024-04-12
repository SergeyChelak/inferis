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
    let Some(component_maze) = storage.get::<Maze>(maze_id) else {
        return Err(EngineError::ComponentNotFound("Maze".to_string()));
    };
    let Some(pos) = storage.get::<Position>(player_id).and_then(|x| Some(x.0)) else {
        return Err(EngineError::ComponentNotFound("Position".to_string()));
    };
    let Some(prev_pos) = storage
        .get::<PrevPosition>(player_id)
        .and_then(|x| Some(x.0))
    else {
        return Err(EngineError::ComponentNotFound("PrevPosition".to_string()));
    };
    let maze = &component_maze.0;
    let (col, row) = (pos.x as usize, pos.y as usize);
    if maze[row][col] == 0 {
        // update with last valid position
        let Some(mut prev_pos) = storage.get_mut::<PrevPosition>(player_id) else {
            return Err(EngineError::ComponentNotFound("PrevPosition".to_string()));
        };
        prev_pos.borrow_mut().0 = pos;
    } else {
        // restore last valid position
        let Some(mut pos) = storage.get_mut::<Position>(player_id) else {
            return Err(EngineError::ComponentNotFound("Position".to_string()));
        };
        pos.borrow_mut().0 = prev_pos;
    }
    Ok(())
}
