mod minimap;
mod raycaster;
use engine::{
    render::WindowCanvas, AssetManager, ComponentStorage, Engine, EngineError, EngineResult,
    EntityID, WindowSize,
};
use minimap::*;

// const FIELD_OF_VIEW: Float = PI / 3.0;
// const MAX_DEPTH: usize = 50;
// const TILE_SIZE: usize = 5;
// const TOL: Float = 1e-5;

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
    let Some(&color) = assets.color("floor") else {
        return Err(EngineError::ResourceNotFound("floor".to_string()));
    };
    context.canvas.set_draw_color(color);
    context.canvas.clear();
    render_sky(&mut context);
    render_minimap(&mut context)?;
    context.canvas.present();
    Ok(())
}

fn render_sky(context: &mut RendererContext) {
    /*
    let w = self.scene_size.width as Float;
       self.offset = 1.5 * angle * w / PI;
       self.offset %= w;
        */
}
