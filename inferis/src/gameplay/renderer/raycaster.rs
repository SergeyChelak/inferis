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
    let (h_depth, h_vec) = cast_horizontal(context, sin, cos);
    let (v_depth, v_vec) = cast_vertical(context, sin, cos);

    let (depth, offset) = if v_depth < h_depth {
        let vertical_y = v_vec.y % 1.0;
        let offset = if cos > 0.0 {
            vertical_y
        } else {
            1.0 - vertical_y
        };
        (v_depth, offset)
    } else {
        let horizontal_x = h_vec.x % 1.0;
        let offset = if sin > 0.0 {
            1.0 - horizontal_x
        } else {
            horizontal_x
        };
        (h_depth, offset)
    };
}

fn collider_check_walls(point: Vec2f, maze: &MazeData) -> Option<String> {
    let Vec2f { x, y } = point;
    if x < 0.0 || y < 0.0 {
        return None;
    }
    let (col, row) = (point.x as usize, point.y as usize);
    let Some(value) = maze.get(row).and_then(|x| x.get(col)) else {
        return None;
    };
    match value {
        1 => Some("wall1".to_string()),
        2 => Some("wall2".to_string()),
        3 => Some("wall3".to_string()),
        4 => Some("wall4".to_string()),
        5 => Some("wall5".to_string()),
        _ => {
            println!("[Renderer] unexpected maze value {value}");
            None
        }
    }
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
