use std::f32::consts::PI;

use engine::{rect::Rect, texture_size, EngineError, EngineResult, Float, Query, Size, Vec2f};

use crate::gameplay::{Angle, HeightShift, Position, ScaleRatio, SpriteTag, TextureID};

use super::{RendererContext, TextureRendererTask};

pub fn render_sprites<'a>(
    context: &mut RendererContext<'a>,
    tasks: &mut Vec<TextureRendererTask<'a>>,
) -> EngineResult<()> {
    let storage = context.storage;
    let player_id = context.player_id;
    let Some(player_pos) = storage.get::<Position>(player_id).map(|x| x.0) else {
        return Err(EngineError::ComponentNotFound("Position".to_string()));
    };
    // player angle must be positive
    let Some(player_angle) =
        storage
            .get::<Angle>(player_id)
            .map(|x| x.0)
            .map(|x| if x < 0.0 { x + 2.0 * PI } else { x })
    else {
        return Err(EngineError::ComponentNotFound("Angle".to_string()));
    };
    let rays_count = context.rays_count();
    let ray_angle_step = context.ray_angle_step();
    let scale = context.scale();
    let screen_distance = context.screen_distance();
    let query = Query::new().with_component::<SpriteTag>();
    for entity_id in storage.fetch_entities(&query) {
        let Some(texture_id_component) = storage.get::<TextureID>(entity_id) else {
            return Err(EngineError::ComponentNotFound("TextureID".to_string()));
        };
        let Some(texture) = context.assets.texture(&texture_id_component.0) else {
            continue;
        };
        let Some(sprite_pos) = storage.get::<Position>(entity_id).map(|x| x.0) else {
            return Err(EngineError::ComponentNotFound("Position".to_string()));
        };
        let sprite_scale = storage
            .get::<ScaleRatio>(entity_id)
            .map(|x| x.0)
            .unwrap_or(1.0);
        let sprite_height_shift = storage
            .get::<HeightShift>(entity_id)
            .map(|x| x.0)
            .unwrap_or(1.0);
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
        let delta_rays = delta / ray_angle_step;
        let x = ((rays_count >> 1) as Float + delta_rays) * scale;
        let norm_distance = vector.hypotenuse() * delta.cos();
        let Size {
            width: w,
            height: h,
        } = texture_size(texture);
        let skip_rendering = {
            let half_width = (w >> 1) as Float;
            x < -half_width
                || x > context.window_size.width as Float + half_width
                || norm_distance < 0.5
        };
        if skip_rendering {
            continue;
        }
        let ratio = w as Float / h as Float;
        let proj = screen_distance / norm_distance * sprite_scale;
        let (proj_width, proj_height) = (proj * ratio, proj);
        let sprite_half_width = 0.5 * proj_width;
        let height_shift = proj_height * sprite_height_shift;
        let sx = x - sprite_half_width;
        let sy = (context.window_size.height as Float - proj_height) * 0.5 + height_shift;
        let task = TextureRendererTask {
            texture,
            source: Rect::new(0, 0, w, h),
            destination: Rect::new(sx as i32, sy as i32, proj_width as u32, proj_height as u32),
            depth: norm_distance,
        };
        tasks.push(task);
    }
    Ok(())
}
