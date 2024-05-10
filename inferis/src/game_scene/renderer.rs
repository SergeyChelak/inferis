use std::{cell::RefCell, collections::HashMap, f32::consts::PI, rc::Rc};

use engine::{
    rect::Rect,
    systems::{GameRendererSystem, RendererEffect, RendererLayers, RendererLayersPtr},
    texture_size, AssetManager, ComponentStorage, EngineError, EngineResult, EntityID, Float,
    SizeU32,
};

use crate::{
    game_scene::fetch_player_id,
    resource::{WORLD_FLOOR_GRADIENT, WORLD_SKY},
};

use super::components;

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;
const MAP_SCALE: u32 = 6;

pub struct RendererSystem {
    layers: RendererLayersPtr,
    texture_size: HashMap<String, SizeU32>,
    // short term cached values
    angle: Float,
    // long term cached values
    player_id: EntityID,
    window_size: SizeU32,
    rays_count: u32,
    ray_angle_step: Float,
    scale: Float,
    screen_distance: Float,
}

impl Default for RendererSystem {
    fn default() -> Self {
        let layers = RendererLayers {
            hud: Vec::with_capacity(200),
            depth: Vec::with_capacity(2000),
            background: Vec::with_capacity(20),
        };
        Self {
            layers: Rc::new(RefCell::new(layers)),
            texture_size: Default::default(),
            angle: Default::default(),
            player_id: Default::default(),
            window_size: Default::default(),
            rays_count: Default::default(),
            ray_angle_step: Default::default(),
            scale: Default::default(),
            screen_distance: Default::default(),
        }
    }
}

impl RendererSystem {
    pub fn new() -> Self {
        Default::default()
    }

    fn update_storage_cache(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        if storage.is_alive(self.player_id) {
            return Ok(());
        }
        self.player_id = fetch_player_id(storage).ok_or(EngineError::unexpected_state(
            "[v2.renderer] player entity not found",
        ))?;
        Ok(())
    }

    fn cache_textures_info(&mut self, asset_manager: &AssetManager) -> EngineResult<()> {
        let ids = asset_manager.texture_ids();
        for id in ids {
            let Some(texture) = asset_manager.texture(&id) else {
                let msg = format!("[v2.renderer] texture id: {}", id);
                return Err(EngineError::TextureNotFound(msg));
            };
            let size = texture_size(texture);
            self.texture_size.insert(id, size);
        }
        Ok(())
    }

    /*
    fn render_player_weapon(&mut self) -> EngineResult<()> {
        let Some(texture_data) = self.texture_data(self.player_id) else {
            return Ok(());
        };
        let Size { width, height } = texture_data.size;

        let Size {
            width: window_width,
            height: window_height,
        } = self.window_size;
        let ratio = height as Float / width as Float;
        let w = (window_width as Float * 0.3) as u32;
        let h = (w as Float * ratio) as u32;

        let destination = Rect::new(
            ((window_width - w) >> 1) as i32,
            (window_height - h) as i32,
            w,
            h,
        );
        let task = TextureRendererTask {
            texture: texture_data.texture,
            source: texture_data.source,
            destination,
            depth: 0.001,
        };
        self.tasks.push(task);
        Ok(())
    }
     */

    // ------------------------------------------------------------------------------------------------------------
    fn render_background(&mut self) -> EngineResult<()> {
        self.render_floor()?;
        self.render_sky()?;
        Ok(())
    }

    fn render_floor(&mut self) -> EngineResult<()> {
        let half_height = self.window_size.height >> 1;
        let destination = Rect::new(0, half_height as i32, self.window_size.width, half_height);
        // gradient floor
        let Some(size) = self.texture_size.get(WORLD_FLOOR_GRADIENT) else {
            return Ok(());
        };
        let source = Rect::new(0, 0, size.width, size.height);
        let mut layers = self.layers.borrow_mut();
        let effect = RendererEffect::Texture {
            asset_id: WORLD_FLOOR_GRADIENT,
            source,
            destination,
        };
        layers.background.push(effect);
        Ok(())
    }

    fn render_sky(&mut self) -> EngineResult<()> {
        let Some(texture_size) = self.texture_size.get(WORLD_SKY) else {
            return Ok(());
        };
        let offset = {
            let w = self.window_size.width as Float;
            let offset = -(1.5 * self.angle * w / PI) % w;
            offset as i32
        };
        let SizeU32 {
            width: w,
            height: h,
        } = *texture_size;
        let source = Rect::new(0, 0, w, h);
        let half_height = self.window_size.height >> 1;
        let destinations = [
            Rect::new(offset, 0, self.window_size.width, half_height),
            Rect::new(
                offset - self.window_size.width as i32,
                0,
                self.window_size.width,
                half_height,
            ),
            Rect::new(
                offset + self.window_size.width as i32,
                0,
                self.window_size.width,
                half_height,
            ),
        ];
        let mut layers = self.layers.borrow_mut();
        for destination in destinations {
            let effect = RendererEffect::Texture {
                asset_id: WORLD_SKY,
                source,
                destination,
            };
            layers.background.push(effect)
        }
        Ok(())
    }
}

impl GameRendererSystem for RendererSystem {
    fn setup(
        &mut self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
        window_size: SizeU32,
    ) -> EngineResult<()> {
        self.update_storage_cache(storage)?;
        self.cache_textures_info(asset_manager)?;
        // precalculated values
        self.window_size = window_size;
        self.rays_count = window_size.width >> 1;
        self.ray_angle_step = FIELD_OF_VIEW / self.rays_count as Float;
        self.scale = window_size.width as Float / self.rays_count as Float;
        self.screen_distance = (window_size.width >> 1) as Float / HALF_FIELD_OF_VIEW.tan();
        println!("[v2.renderer] setup ok");
        Ok(())
    }

    fn render(
        &mut self,
        frames: usize,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<RendererLayersPtr> {
        self.update_storage_cache(storage)?;

        // prefetch
        self.angle = storage
            .get::<components::Angle>(self.player_id)
            .map(|x| x.0)
            .ok_or(EngineError::component_not_found("[v2.renderer] angle"))?;

        self.layers.borrow_mut().clear();
        self.render_background()?;
        Ok(self.layers.clone())
    }
}
