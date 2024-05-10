use std::{collections::HashMap, f32::consts::PI};

use engine::{
    systems::{vec_ptr, GameRendererSystem, RendererEffect, VecPtr},
    texture_size, AssetManager, ComponentStorage, EngineError, EngineResult, EntityID, Float,
    SizeU32,
};

use crate::game_scene::fetch_player_id;

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;
const MAP_SCALE: u32 = 6;

#[derive(Default)]
pub struct RendererSystem {
    effect_buffer: VecPtr<RendererEffect>,
    texture_size: HashMap<String, SizeU32>,
    player_id: EntityID,
    window_size: SizeU32,
    rays_count: u32,
    ray_angle_step: Float,
    scale: Float,
    screen_distance: Float,
}

impl RendererSystem {
    pub fn new() -> Self {
        Self {
            effect_buffer: vec_ptr(1000),
            ..Default::default()
        }
    }

    fn cache_player_id(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
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
}

impl GameRendererSystem for RendererSystem {
    fn setup(
        &mut self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
        window_size: SizeU32,
    ) -> EngineResult<()> {
        self.cache_player_id(storage)?;
        self.cache_textures_info(asset_manager)?;
        // precalculated values
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
    ) -> EngineResult<VecPtr<RendererEffect>> {
        let mut buffer = self.effect_buffer.borrow_mut();
        buffer.clear();
        // TODO: ...
        Ok(self.effect_buffer.clone())
    }
}
