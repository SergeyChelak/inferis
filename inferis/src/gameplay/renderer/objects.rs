use std::cmp::Ordering;

use engine::{rect::Rect, texture_size, EngineError, EngineResult, Float, Size, Vec2f};

use crate::gameplay::{
    components::{Angle, Maze, MazeData, Position},
    ray_caster::*,
};

use super::{sprites::render_sprites, RendererContext, TextureRendererTask, HALF_FIELD_OF_VIEW};

pub fn render_game_objects(context: &mut RendererContext) -> EngineResult<()> {
    let mut tasks = Vec::<TextureRendererTask>::new();
    render_walls(context, &mut tasks)?;
    render_sprites(context, &mut tasks)?;
    tasks.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(Ordering::Equal));
    for task in tasks {
        context
            .canvas
            .copy(task.texture, task.source, task.destination)
            .map_err(|e| EngineError::Sdl(e.to_string()))?;
    }
    Ok(())
}

fn render_walls<'a>(
    context: &mut RendererContext<'a>,
    tasks: &mut Vec<TextureRendererTask<'a>>,
) -> EngineResult<()> {
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
    let height = context.window_size.height as Float;
    // ray
    let mut ray_angle = angle - HALF_FIELD_OF_VIEW;
    let rays_count = context.rays_count();
    let ray_angle_step = context.ray_angle_step();
    // distance
    let screen_distance = context.screen_distance();
    let scale = context.scale();
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
        let Size {
            width: w,
            height: h,
        } = texture_size(texture);
        let src = Rect::new(
            (result.offset * (w as Float - image_width as Float)) as i32,
            0,
            image_width,
            h,
        );
        let task = TextureRendererTask {
            texture,
            source: src,
            destination: dst,
            depth,
        };
        tasks.push(task);

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
