use std::{
    borrow::BorrowMut,
    cell::{Ref, RefMut},
    f32::consts::PI,
};

use engine::{
    rect::Rect, render::Texture, texture_size, EngineError, EngineResult, EntityID, Float, Query,
    Size, SizeU32, Vec2f,
};

use crate::gameplay::{
    Angle, AnimationData, HeightShift, Position, ScaleRatio, SpriteTag, TextureID,
};

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
        let Some(texture_data) = TextureData::new(context, entity_id) else {
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
        } = texture_data.size;
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
            texture: texture_data.texture,
            source: texture_data.source,
            destination: Rect::new(sx as i32, sy as i32, proj_width as u32, proj_height as u32),
            depth: norm_distance,
        };
        tasks.push(task);
    }
    Ok(())
}

struct TextureData<'a> {
    size: SizeU32,
    source: Rect,
    texture: &'a Texture<'a>,
}

impl<'a> TextureData<'a> {
    fn new(context: &mut RendererContext<'a>, entity_id: EntityID) -> Option<Self> {
        let storage = context.storage;
        if let Some(mut animation_data) = storage.get_mut::<AnimationData>(entity_id) {
            if animation_data.frame_counter >= animation_data.target_frames {
                // TODO: refactor render implementation to delete animation from entity
                return None;
            }
            Self::with_animation(context, &mut animation_data)
        } else if let Some(texture_id_component) = storage.get::<TextureID>(entity_id) {
            Self::with_texture(context, texture_id_component)
        } else {
            None
        }
    }

    fn with_animation(
        context: &mut RendererContext<'a>,
        animation_data: &mut RefMut<AnimationData>,
    ) -> Option<Self> {
        let params = context.assets.animation(&animation_data.animation_id)?;
        let texture = context.assets.texture(&params.texture_id)?;
        let size = texture_size(texture);
        let frame_size = Size {
            width: size.width / params.frames_count as u32,
            height: size.height,
        };
        let index = (animation_data.frame_counter / params.duration as usize) % params.frames_count;
        let source = Rect::new(
            frame_size.width as i32 * index as i32,
            0,
            frame_size.width,
            frame_size.height,
        );
        animation_data.borrow_mut().frame_counter += 1;
        Some(Self {
            size: frame_size,
            source,
            texture,
        })
    }

    fn with_texture(
        context: &mut RendererContext<'a>,
        texture_id_component: Ref<TextureID>,
    ) -> Option<Self> {
        let texture = context.assets.texture(&texture_id_component.0)?;
        let size = texture_size(texture);
        let source = Rect::new(0, 0, size.width, size.height);
        Some(Self {
            size,
            source,
            texture,
        })
    }
}
