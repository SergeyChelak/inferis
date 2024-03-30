use engine::{
    pixels::Color,
    rect::{Point, Rect},
    EngineError, EngineResult, Float,
};

use crate::gameplay::components::{Angle, Maze, Position};

use super::RendererContext;

pub fn render_minimap(context: &mut RendererContext) -> EngineResult<()> {
    render_walls(context)?;
    render_player_position(context)
}

const MAP_SCALE: u32 = 6;

fn render_walls(context: &mut RendererContext) -> EngineResult<()> {
    let Some(maze_comp) = context.storage.get::<Maze>(context.maze_id) else {
        return Ok(());
    };
    let maze = &maze_comp.0;
    context.canvas.set_draw_color(Color::WHITE);
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
            context
                .canvas
                .fill_rect(rect)
                .map_err(|e| EngineError::Sdl(e.to_string()))?
        }
    }
    Ok(())
}

fn render_player_position(context: &mut RendererContext) -> EngineResult<()> {
    let storage = context.storage;
    let canvas = &mut context.canvas;
    let player_id = context.player_id;
    let Some(pos) = storage.get::<Position>(player_id).and_then(|x| Some(x.0)) else {
        return Err(EngineError::ComponentNotFound("Position".to_string()));
    };
    let Some(angle) = storage.get::<Angle>(player_id).and_then(|x| Some(x.0)) else {
        return Err(EngineError::ComponentNotFound("Angle".to_string()));
    };
    let (x, y) = (
        (pos.x * MAP_SCALE as Float) as i32,
        (pos.y * MAP_SCALE as Float) as i32,
    );

    let size = MAP_SCALE - 2;
    let rect = Rect::new(x - (size >> 1) as i32, y - (size >> 1) as i32, size, size);
    canvas.set_draw_color(Color::RED);
    canvas
        .fill_rect(rect)
        .map_err(|e| EngineError::Sdl(e.to_string()))?;

    let length = 1.5 * MAP_SCALE as Float;
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
