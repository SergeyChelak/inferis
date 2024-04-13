mod minimap;
mod objects;
mod sprites;
use std::f32::consts::PI;

use engine::{
    rect::Rect,
    render::{Texture, WindowCanvas},
    AssetManager, ComponentStorage, Engine, EngineError, EngineResult, EntityID, Float, SizeU32,
    Vec2f,
};
use minimap::render_minimap;
use objects::*;

use super::{Angle, Position};

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
    render_game_objects(&mut context)?;
    render_minimap(&mut context)?;
    Ok(())
}

pub struct Renderer<'a> {
    storage: &'a mut ComponentStorage,
    engine: &'a mut dyn Engine,
    assets: &'a AssetManager<'a>,
    player_id: EntityID,
    maze_id: EntityID,
    player_position: Option<Vec2f>,
    player_angle: Option<Float>,
    // cached values
    window_size: SizeU32,
    rays_count: u32,
    ray_angle_step: Float,
    scale: Float,
    screen_distance: Float,
}

impl<'a> Renderer<'a> {
    pub fn new(
        storage: &'a mut ComponentStorage,
        engine: &'a mut dyn Engine,
        assets: &'a AssetManager,
        player_id: EntityID,
        maze_id: EntityID,
    ) -> Self {
        let window_size = engine.window_size();
        let rays_count = window_size.width >> 1;
        let ray_angle_step = FIELD_OF_VIEW / rays_count as Float;
        let scale = window_size.width as Float / rays_count as Float;
        let screen_distance = (window_size.width >> 1) as Float / HALF_FIELD_OF_VIEW.tan();
        Self {
            storage,
            engine,
            assets,
            player_id,
            maze_id,
            player_position: None,
            player_angle: None,
            window_size,
            rays_count,
            ray_angle_step,
            scale,
            screen_distance,
        }
    }

    pub fn render(&mut self) -> EngineResult<()> {
        self.fetch_common_info()?;
        self.render_background()?;
        Ok(())
    }

    fn fetch_common_info(&mut self) -> EngineResult<()> {
        let Some(player_pos) = self.storage.get::<Position>(self.player_id).map(|x| x.0) else {
            return Err(EngineError::ComponentNotFound("Position".to_string()));
        };
        // player angle must be positive
        let Some(player_angle) = self
            .storage
            .get::<Angle>(self.player_id)
            .map(|x| x.0)
            .map(|x| if x < 0.0 { x + 2.0 * PI } else { x })
        else {
            return Err(EngineError::ComponentNotFound("Angle".to_string()));
        };
        self.player_position = Some(player_pos);
        self.player_angle = Some(player_angle);
        Ok(())
    }

    #[inline]
    fn canvas(&mut self) -> &mut WindowCanvas {
        self.engine.canvas()
    }

    fn render_background(&mut self) -> EngineResult<()> {
        self.render_sky()?;
        self.render_floor(true)?;
        Ok(())
    }

    fn render_floor(&mut self, gradient: bool) -> EngineResult<()> {
        let half_height = self.window_size.height >> 1;
        let dst = Rect::new(0, half_height as i32, self.window_size.width, half_height);
        if gradient {
            // gradient floor
            let Some(texture) = self.assets.texture("floor_grad") else {
                return Err(EngineError::TextureNotFound("floor_grad".to_string()));
            };
            let src = {
                let query = texture.query();
                let (w, h) = (query.width, query.height);
                Rect::new(0, 0, w, h)
            };
            self.canvas()
                .copy(texture, src, dst)
                .map_err(|e| EngineError::Sdl(e.to_string()))?;
        } else {
            // solid floor
            let Some(&color) = self.assets.color("floor") else {
                return Err(EngineError::ResourceNotFound("floor".to_string()));
            };
            self.canvas().set_draw_color(color);
            self.canvas()
                .fill_rect(dst)
                .map_err(|e| EngineError::Sdl(e.to_string()))?;
        }
        Ok(())
    }

    fn render_sky(&mut self) -> EngineResult<()> {
        let Some(texture) = self.assets.texture("sky") else {
            return Err(EngineError::TextureNotFound("sky".to_string()));
        };
        let Some(angle) = self.player_angle else {
            return Err(EngineError::ComponentNotFound("Angle".to_string()));
        };
        let w = self.window_size.width as Float;
        let offset = -(1.5 * angle * w / PI) % w;
        let offset = offset as i32;
        let (w, h) = {
            let query = texture.query();
            (query.width, query.height)
        };
        let src = Rect::new(0, 0, w, h);
        let half_height = self.window_size.height >> 1;
        let dst = Rect::new(offset, 0, self.window_size.width, half_height);
        self.canvas()
            .copy(texture, src, dst)
            .map_err(|e| EngineError::Sdl(e.to_string()))?;
        let dst = Rect::new(
            offset - self.window_size.width as i32,
            0,
            self.window_size.width,
            half_height,
        );
        self.canvas()
            .copy(texture, src, dst)
            .map_err(|e| EngineError::Sdl(e.to_string()))?;
        let dst = Rect::new(
            offset + self.window_size.width as i32,
            0,
            self.window_size.width,
            half_height,
        );
        self.canvas()
            .copy(texture, src, dst)
            .map_err(|e| EngineError::Sdl(e.to_string()))?;
        Ok(())
    }
}
