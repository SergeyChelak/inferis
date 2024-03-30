use std::f32::consts::PI;

use engine::{rect::Rect, EngineError, EngineResult, Float, Vec2f};

use crate::gameplay::{
    components::{Angle, Maze, MazeData, Position},
    ray_caster::*,
};

use super::RendererContext;

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;

pub fn render_game_objects(context: &mut RendererContext) -> EngineResult<()> {
    let storage = context.storage;
    let player_id = context.player_id;
    let Some(pos) = storage.get::<Position>(player_id).and_then(|x| Some(x.0)) else {
        return Err(EngineError::ComponentNotFound("Position".to_string()));
    };
    let Some(angle) = storage.get::<Angle>(player_id).and_then(|x| Some(x.0)) else {
        return Err(EngineError::ComponentNotFound("Angle".to_string()));
    };
    let Some(component_maze) = context.storage.get::<Maze>(context.maze_id) else {
        return Err(EngineError::ComponentNotFound("Maze".to_string()));
    };
    // dims
    let width = context.window_size.width;
    let width_float = width as Float;
    let height = context.window_size.height as Float;
    // ray
    let mut ray_angle = angle - HALF_FIELD_OF_VIEW;
    let rays_count = width >> 1;
    let ray_angle_step = FIELD_OF_VIEW / rays_count as Float;
    // distance
    let scale = width_float / rays_count as Float;
    let screen_distance = 0.5 * width_float * HALF_FIELD_OF_VIEW.tan();
    let image_width = scale as u32;

    let check = |point: Vec2f| wall_texture(point, &component_maze.0);
    for ray in 0..rays_count {
        let result = ray_cast(pos, ray_angle, &check);
        let Some(texture) = result.value.and_then(|key| context.assets.texture(key)) else {
            continue;
        };
        // get rid of fishbowl effect
        let depth = result.depth * (angle - ray_angle).cos();
        let projected_height = screen_distance / (depth + RAY_CASTER_TOL);

        let x = (ray as Float * scale) as i32;
        let y = (0.5 * (height - projected_height)) as i32;

        let dst = Rect::new(x, y, image_width, projected_height as u32);
        let (w, h) = {
            let query = texture.query();
            (query.width, query.height)
        };
        let src = Rect::new(
            (result.offset * (w as Float - image_width as Float)) as i32,
            0,
            image_width,
            h,
        );
        context
            .canvas
            .copy(texture, src, dst)
            .map_err(|e| EngineError::Sdl(e.to_string()))?;

        ray_angle += ray_angle_step;
    }
    Ok(())
}

fn wall_texture(point: Vec2f, maze: &MazeData) -> Option<&str> {
    let Vec2f { x, y } = point;
    if x < 0.0 || y < 0.0 {
        return None;
    }
    let (col, row) = (point.x as usize, point.y as usize);
    let Some(value) = maze.get(row).and_then(|x| x.get(col)) else {
        return None;
    };
    match value {
        1 => Some("wall1"),
        2 => Some("wall2"),
        3 => Some("wall3"),
        4 => Some("wall4"),
        5 => Some("wall5"),
        _ => None,
    }
}
