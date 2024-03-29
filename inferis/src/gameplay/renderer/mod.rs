pub mod minimap;
use engine::{AssetManager, ComponentStorage, Engine, EngineError, EngineResult, EntityID};
pub use minimap::*;

// const FIELD_OF_VIEW: Float = PI / 3.0;
// const MAX_DEPTH: usize = 50;
// const TILE_SIZE: usize = 5;
// const TOL: Float = 1e-5;

pub fn render_scene(
    storage: &ComponentStorage,
    engine: &mut dyn Engine,
    assets: &AssetManager,
    player_id: EntityID,
) -> EngineResult<()> {
    let canvas = engine.canvas();
    let Some(&color) = assets.color("floor") else {
        return Err(EngineError::ResourceNotFound("floor".to_string()));
    };
    canvas.set_draw_color(color);
    canvas.clear();
    render_minimap(storage, player_id, canvas)?;
    canvas.present();
    Ok(())
}
