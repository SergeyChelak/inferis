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
    let width = context.window_size.width;
    let height = context.window_size.height as Float;
    let screen_distance = width as Float * 0.5 * HALF_FIELD_OF_VIEW.tan();
    let rays = width >> 1;
    let scale = width as Float / rays as Float;
    let mut ray_angle = angle - HALF_FIELD_OF_VIEW;
    let ray_angle_step = FIELD_OF_VIEW / rays as Float;
    let check = |point: Vec2f| wall_texture(point, &component_maze.0);
    for ray in 0..rays {
        let result = ray_cast(pos, ray_angle, &check);
        let Some(texture) = result.value.and_then(|key| context.assets.texture(&key)) else {
            continue;
        };
        // get rid of fishbowl effect
        let depth = result.depth * (angle - ray_angle).cos();
        let projected_height = screen_distance / (depth + RAY_CASTER_TOL);

        let x = (ray as Float * scale) as i32;
        let y = (0.5 * (height - projected_height)) as i32;
        // offset
        // width = scale as u32
        // projected_height
        // texture id

        let dst = Rect::new(x, y, width, projected_height as u32);
        let (w, h) = {
            let query = texture.query();
            (query.width, query.height)
        };
        let src = Rect::new(
            (result.offset * (w as Float - width as Float)) as i32,
            0,
            width,
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

/*
    DrawCommand::Texture {
        depth,
        x,
        y,
        offset,
        width,
        projected_height,
        texture_id,
    } => {
        let dst = Rect::new(x, y, width, projected_height);
        let Some(texture) = textures.get(&texture_id) else {
            // draw gray-scale bars in case of missing texture
            let clr = (255.0 / (1.0 + depth.powi(5) * 0.00002)) as u8;
            self.canvas.set_draw_color(Color::RGB(clr, clr, clr));
            self.canvas.draw_rect(dst)?;
            continue;
        };
        let query = texture.query();
        let (w, h) = (query.width, query.height);
        let src =
            Rect::new((offset * (w as Float - width as Float)) as i32, 0, width, h);
        self.canvas.copy(texture, src, dst)?;
    }
*/

fn wall_texture(point: Vec2f, maze: &MazeData) -> Option<String> {
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
        _ => None,
    }
}
