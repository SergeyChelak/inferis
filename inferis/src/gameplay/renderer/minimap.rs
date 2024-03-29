// use std::f32::consts::PI;

use engine::{
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
    ComponentStorage, EngineError, EngineResult, EntityID, Float, Query,
};

use crate::gameplay::components::{Angle, Maze, Position};

pub fn render_minimap(
    storage: &ComponentStorage,
    player_id: EntityID,
    canvas: &mut WindowCanvas,
) -> EngineResult<()> {
    render_walls(storage, canvas)?;
    render_player_position(storage, player_id, canvas)
}

const MAP_SCALE: u32 = 6;

fn render_walls(storage: &ComponentStorage, canvas: &mut WindowCanvas) -> EngineResult<()> {
    let query_map = Query::new().with_component::<Maze>();
    let Some(maze_comp) = storage
        .fetch_entities(&query_map)
        .first()
        .and_then(|id| storage.get::<Maze>(*id))
    else {
        // ???
        return Ok(());
    };
    let maze = &maze_comp.0;
    canvas.set_draw_color(Color::WHITE);
    for (row, vector) in maze.iter().enumerate() {
        for (col, value) in vector.iter().enumerate() {
            if *value == 0 {
                continue;
            }
            let rect = Rect::new(
                col as i32 * MAP_SCALE as i32,
                row as i32 * MAP_SCALE as i32,
                MAP_SCALE,
                MAP_SCALE,
            );
            canvas
                .fill_rect(rect)
                .map_err(|e| EngineError::Sdl(e.to_string()))?
        }
    }
    Ok(())
}

fn render_player_position(
    storage: &ComponentStorage,
    player_id: EntityID,
    canvas: &mut WindowCanvas,
) -> EngineResult<()> {
    let Some(pos) = storage.get::<Position>(player_id) else {
        return Err(EngineError::ComponentNotFound("Position".to_string()));
    };
    let Some(angle_comp) = storage.get::<Angle>(player_id) else {
        return Err(EngineError::ComponentNotFound("Angle".to_string()));
    };
    let (x, y) = (pos.0.x as i32, pos.0.y as i32);
    let rect = Rect::new(
        x - (MAP_SCALE >> 1) as i32,
        y - (MAP_SCALE >> 1) as i32,
        MAP_SCALE,
        MAP_SCALE,
    );
    canvas.set_draw_color(Color::RED);
    canvas
        .fill_rect(rect)
        .map_err(|e| EngineError::Sdl(e.to_string()))?;

    let length = 1.5 * MAP_SCALE as Float;
    let angle = angle_comp.0;
    canvas
        .draw_line(
            Point::new(x, y),
            Point::new(
                x + (length * angle.cos()) as i32,
                y + (length * angle.sin()) as i32,
            ),
        )
        .map_err(|e| EngineError::Sdl(e.to_string()))
}
