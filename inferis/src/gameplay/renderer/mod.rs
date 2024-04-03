mod background;
mod minimap;
mod objects;
use background::*;

use engine::{
    pixels::Color, render::WindowCanvas, AssetManager, ComponentStorage, Engine, EngineResult,
    EntityID, WindowSize,
};
use minimap::*;
use objects::*;

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
    context.canvas.set_draw_color(Color::BLACK);
    context.canvas.clear();
    render_background(&mut context)?;
    render_game_objects(&mut context)?;
    render_minimap(&mut context)?;
    context.canvas.present();
    Ok(())
}
