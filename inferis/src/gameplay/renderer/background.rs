use std::f32::consts::PI;

use engine::{rect::Rect, EngineError, EngineResult, Float};

use crate::gameplay::components::Angle;

use super::RendererContext;

pub fn render_background(context: &mut RendererContext) -> EngineResult<()> {
    render_sky(context)?;
    render_floor(context, true)?;
    Ok(())
}

fn render_floor(context: &mut RendererContext, gradient: bool) -> EngineResult<()> {
    let window_size = context.window_size;
    let half_height = window_size.height >> 1;
    let dst = Rect::new(0, half_height as i32, window_size.width, half_height);
    if gradient {
        // gradient floor
        let Some(texture) = context.assets.texture("floor_grad") else {
            return Err(EngineError::TextureNotFound("floor_grad".to_string()));
        };
        let src = {
            let query = texture.query();
            let (w, h) = (query.width, query.height);
            Rect::new(0, 0, w, h)
        };
        context
            .canvas
            .copy(texture, src, dst)
            .map_err(|e| EngineError::Sdl(e.to_string()))?;
    } else {
        // solid floor
        let Some(&color) = context.assets.color("floor") else {
            return Err(EngineError::ResourceNotFound("floor".to_string()));
        };
        context.canvas.set_draw_color(color);
        context
            .canvas
            .fill_rect(dst)
            .map_err(|e| EngineError::Sdl(e.to_string()))?;
    }
    Ok(())
}

fn render_sky(context: &mut RendererContext) -> EngineResult<()> {
    let Some(texture) = context.assets.texture("sky") else {
        return Err(EngineError::TextureNotFound("sky".to_string()));
    };
    let Some(angle) = context
        .storage
        .get::<Angle>(context.player_id)
        .and_then(|x| Some(x.0))
    else {
        return Err(EngineError::ComponentNotFound("Angle".to_string()));
    };
    let window_size = context.window_size;
    let w = window_size.width as Float;
    let offset = -(1.5 * angle * w / PI) % w;
    let offset = offset as i32;
    let (w, h) = {
        let query = texture.query();
        (query.width, query.height)
    };
    let src = Rect::new(0, 0, w, h);
    let half_height = window_size.height >> 1;
    let dst = Rect::new(offset, 0, window_size.width, half_height);
    context
        .canvas
        .copy(texture, src, dst)
        .map_err(|e| EngineError::Sdl(e.to_string()))?;
    let dst = Rect::new(
        offset - window_size.width as i32,
        0,
        window_size.width,
        half_height,
    );
    context
        .canvas
        .copy(texture, src, dst)
        .map_err(|e| EngineError::Sdl(e.to_string()))?;
    let dst = Rect::new(
        offset + window_size.width as i32,
        0,
        window_size.width,
        half_height,
    );
    context
        .canvas
        .copy(texture, src, dst)
        .map_err(|e| EngineError::Sdl(e.to_string()))?;
    Ok(())
}
