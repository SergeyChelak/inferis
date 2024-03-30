use std::f32::consts::PI;

use engine::{EngineError, EngineResult, Float, Vec2f};

use crate::gameplay::components::{Angle, Maze, MazeData, Position};

use super::RendererContext;

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;
const TOL: Float = 1e-5;
const MAX_DEPTH: usize = 50;

struct RayCastContext<'a> {
    maze: &'a MazeData,
    pos: Vec2f,
    tile: Vec2f,
}

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
    let mut ray_cast_context = RayCastContext {
        maze: &component_maze.0,
        pos,
        tile: pos.floor(),
    };

    let rays = context.window_size.width >> 1;
    let mut ray_angle = angle - HALF_FIELD_OF_VIEW;
    let ray_angle_step = FIELD_OF_VIEW / rays as Float;
    for _ in 0..rays {
        ray_cast(&mut ray_cast_context, ray_angle);
        ray_angle += ray_angle_step;
    }
    Ok(())
}

fn ray_cast(context: &mut RayCastContext, ray_angle: Float) {
    let sin = ray_angle.sin();
    let cos = ray_angle.cos();
    let _ = cast_horizontal(context, sin, cos);
    let _ = cast_vertical(context, sin, cos);
}

fn cast_horizontal(context: &mut RayCastContext, sin: Float, cos: Float) -> (Float, Vec2f) {
    // horizontals
    let tile = context.tile;
    let pos = context.pos;
    let (mut y, dy) = if sin > 0.0 {
        (tile.y + 1.0, 1.0)
    } else {
        (tile.y - TOL, -1.0)
    };
    let mut depth = (y - pos.y) / sin;
    let mut x = pos.x + depth * cos;
    let depth_delta = dy / sin;
    let dx = depth_delta * cos;
    for _ in 0..MAX_DEPTH {
        // let point = Vec2f::new(horizontal_x, horizontal_y);
        // if map.has_collision(point) {
        //     texture_id_horizontal = map.texture_id(point);
        //     break;
        // }
        x += dx;
        y += dy;
        depth += depth_delta;
    }
    (depth, Vec2f::new(x, y))
}

fn cast_vertical(context: &mut RayCastContext, sin: Float, cos: Float) -> (Float, Vec2f) {
    // verticals
    let tile = context.tile;
    let pos = context.pos;

    let (mut x, dx) = if cos > 0.0 {
        (tile.x + 1.0, 1.0)
    } else {
        (tile.x - TOL, -1.0)
    };
    let mut depth = (x - pos.x) / cos;
    let mut y = pos.y + depth * sin;
    let depth_delta = dx / cos;
    let dy = depth_delta * sin;
    for _ in 0..MAX_DEPTH {
        // let point = Float2d::new(vertical_x, vertical_y);
        // if map.has_collision(point) {
        //     texture_id_vertical = map.texture_id(point);
        //     break;
        // }
        x += dx;
        y += dy;
        depth += depth_delta;
    }
    (depth, Vec2f::new(x, y))
}
