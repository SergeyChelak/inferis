mod minimap;
mod objects;
use std::f32::consts::PI;

use engine::{
    pixels::Color, rect::Rect, render::WindowCanvas, AssetManager, ComponentStorage, Engine,
    EngineError, EngineResult, EntityID, Float, WindowSize,
};
use minimap::*;
use objects::*;

use super::components::Angle;

pub struct RendererContext<'a> {
    storage: &'a ComponentStorage,
    canvas: &'a mut WindowCanvas,
    assets: &'a AssetManager<'a>,
    window_size: WindowSize,
    player_id: EntityID,
    maze_id: EntityID,
}

pub fn render_scene(
    storage: &ComponentStorage,
    engine: &mut dyn Engine,
    assets: &AssetManager,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    let mut context = {
        let window_size = engine.window_size();
        let canvas = engine.canvas();
        RendererContext {
            storage,
            canvas,
            assets,
            window_size,
            player_id,
            maze_id,
        }
    };
    // let Some(&color) = assets.color("floor") else {
    //     return Err(EngineError::ResourceNotFound("floor".to_string()));
    // };
    context.canvas.set_draw_color(Color::BLACK);
    context.canvas.clear();
    render_sky(&mut context)?;
    render_floor(&mut context, true)?;
    render_game_objects(&mut context)?;
    render_minimap(&mut context)?;
    context.canvas.present();
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