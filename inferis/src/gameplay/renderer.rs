use engine::{
    pixels::Color, rect::Rect, AssetManager, ComponentStorage, Engine, EngineError, EngineResult,
    EntityID,
};

use super::components::Position;

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
    // draw player rect
    {
        let Some(pos) = storage.get::<Position>(player_id) else {
            return Err(EngineError::ComponentNotFound("Position".to_string()));
        };
        // let scale = 10.0;
        let rect = Rect::new(pos.0.x as i32, pos.0.y as i32, 10, 10);
        canvas.set_draw_color(Color::RED);
        canvas
            .fill_rect(rect)
            .map_err(|e| EngineError::Sdl(e.to_string()))?
    }
    canvas.present();
    Ok(())
}
