use std::f32::consts::PI;

use engine::{rect::Rect, render::Texture, EngineError, EngineResult, Float, Query, Size, Vec2f};

use crate::gameplay::{
    components::{Angle, Maze, MazeData, Position, SpriteTag, TextureID},
    ray_caster::*,
};

use super::RendererContext;

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;

pub fn render_game_objects(context: &mut RendererContext) -> EngineResult<()> {
    render_walls(context)?;
    // render_sprites(context)?;
    Ok(())
}

fn render_sprites(context: &mut RendererContext) -> EngineResult<()> {
    let storage = context.storage;

    let player_id = context.player_id;
    let Some(player_pos) = storage.get::<Position>(player_id).and_then(|x| Some(x.0)) else {
        return Err(EngineError::ComponentNotFound("Position".to_string()));
    };
    let Some(player_angle) = storage.get::<Angle>(player_id).and_then(|x| Some(x.0)) else {
        return Err(EngineError::ComponentNotFound("Angle".to_string()));
    };

    let query = Query::new().with_component::<SpriteTag>();
    for entity_id in storage.fetch_entities(&query) {
        let Some(texture_id_component) = storage.get::<TextureID>(entity_id) else {
            return Err(EngineError::ComponentNotFound("TextureID".to_string()));
        };
        let Some(texture) = context.assets.texture(&texture_id_component.0) else {
            continue;
        };
        let Some(sprite_pos) = storage.get::<Position>(entity_id).and_then(|x| Some(x.0)) else {
            return Err(EngineError::ComponentNotFound("Position".to_string()));
        };

        let vector = sprite_pos - player_pos;

        let delta = {
            let Vec2f { x: dx, y: dy } = vector;
            let theta = dy.atan2(dx);
            let value = theta - player_angle;
            if dx > 0.0 && player_angle > PI || dx < 0.0 && dy < 0.0 {
                value + 2.0 * PI
            } else {
                value
            }
        };

        // TODO: move to function (ray_angle)
        let width = context.window_size.width;
        let rays_count = width >> 1;
        let delta_rays = FIELD_OF_VIEW / rays_count as Float;

        let scale = width as Float / rays_count as Float;
        let x = ((width >> 2) as Float + delta_rays) * scale;

        let dist = vector.dist();
        let norm_dist = dist * delta.cos();
        let Size {
            width: w,
            height: h,
        } = texture_size(texture);
        let half_width = (w >> 1) as Float;
        let w = w as Float;
        // if !(-half_width < x && x < (w + half_width) && norm_dist > 0.5) {
        //     return Ok(());
        // }

        // ????
        let sprite_scale = 1.0;
        let ratio = w / h as Float;

        let screen_dist = half_width / HALF_FIELD_OF_VIEW.tan();
        let proj = screen_dist / norm_dist * sprite_scale;
        let (proj_width, proj_height) = (proj * ratio, proj);

        let sprite_half_width = 0.5 * proj_width;
        let sx = x - sprite_half_width;
        let sy = (context.window_size.height >> 1) as Float - proj_height * 0.5;

        let dst = Rect::new(sx as i32, sy as i32, proj_width as u32, proj_height as u32);
        let src = Rect::new(0, 0, w as u32, h);
        context
            .canvas
            .copy(texture, src, dst)
            .map_err(|e| EngineError::Sdl(e.to_string()))?;
        /*

        */
    }
    Ok(())
}

fn render_walls(context: &mut RendererContext) -> EngineResult<()> {
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
    let rays_count = context.rays_count();
    let ray_angle_step = FIELD_OF_VIEW / rays_count as Float;
    // distance
    let scale = width_float / rays_count as Float;
    let screen_distance = 1.3 * width_float * HALF_FIELD_OF_VIEW.tan();
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

fn texture_size(texture: &Texture) -> Size<u32> {
    let query = texture.query();
    Size {
        width: query.width,
        height: query.height,
    }
}
