// use std::f32::consts::PI;

use engine::{
    pixels::Color, rect::Rect, render::WindowCanvas, AssetManager, ComponentStorage, Engine,
    EngineError, EngineResult, EntityID, Query,
};

use super::components::{Maze, Position};

// const FIELD_OF_VIEW: Float = PI / 3.0;
// const MAX_DEPTH: usize = 50;
// const TILE_SIZE: usize = 5;
// const TOL: Float = 1e-5;

pub fn render_scene(
    storage: &ComponentStorage,
    engine: &mut dyn Engine,
    assets: &AssetManager,
    player_id: EntityID,
) -> EngineResult<()> {
    let canvas = engine.canvas();
    let Some(&color) = assets.color("floor") else {
        return Err(EngineError::ResourceNotFound("floor".to_string()));
    };
    canvas.set_draw_color(color);
    canvas.clear();
    render_map(storage, canvas)?;
    // draw player rect
    {
        let Some(pos) = storage.get::<Position>(player_id) else {
            return Err(EngineError::ComponentNotFound("Position".to_string()));
        };
        // let scale = 10.0;
        let rect = Rect::new(pos.0.x as i32, pos.0.y as i32, 10, 10);
        canvas.set_draw_color(Color::RED);
        canvas
            .fill_rect(rect)
            .map_err(|e| EngineError::Sdl(e.to_string()))?
    }
    canvas.present();
    Ok(())
}

// map drawing
const MAP_SCALE: u32 = 20;
fn render_map(storage: &ComponentStorage, canvas: &mut WindowCanvas) -> EngineResult<()> {
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
