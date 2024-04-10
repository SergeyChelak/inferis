mod background;
mod minimap;
mod objects;
mod sprites;
use std::f32::consts::PI;

use background::render_background;

use engine::{
    pixels::Color,
    rect::Rect,
    render::{Texture, WindowCanvas},
    AssetManager, ComponentStorage, Engine, EngineResult, EntityID, Float, SizeU32,
};
use minimap::render_minimap;
use objects::*;

pub const FIELD_OF_VIEW: Float = PI / 3.0;
pub const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;

pub struct RendererContext<'a> {
    storage: &'a ComponentStorage,
    canvas: &'a mut WindowCanvas,
    assets: &'a AssetManager<'a>,
    window_size: SizeU32,
    player_id: EntityID,
    maze_id: EntityID,
}

impl<'a> RendererContext<'a> {
    pub fn rays_count(&self) -> u32 {
        let width = self.window_size.width;
        width >> 1
    }

    pub fn ray_angle_step(&self) -> Float {
        FIELD_OF_VIEW / self.rays_count() as Float
    }

    pub fn scale(&self) -> Float {
        let width = self.window_size.width as Float;
        width / self.rays_count() as Float
    }

    pub fn screen_distance(&self) -> Float {
        let width = (self.window_size.width >> 1) as Float;
        width / HALF_FIELD_OF_VIEW.tan()
    }
}

pub struct TextureRendererTask<'a> {
    texture: &'a Texture<'a>,
    source: Rect,
    destination: Rect,
    depth: Float,
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
    context.canvas.set_draw_color(Color::BLACK);
    context.canvas.clear();
    render_background(&mut context)?;
    render_game_objects(&mut context)?;
    render_minimap(&mut context)?;
    context.canvas.present();
    Ok(())
}
