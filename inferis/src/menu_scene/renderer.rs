use std::{cell::RefCell, collections::HashMap, rc::Rc};

use engine::{
    rect::Rect,
    systems::{GameRendererSystem, RendererEffect, RendererLayers, RendererLayersPtr},
    EngineResult, SizeU32,
};

use crate::resource::MENU_BACKGROUND;

pub struct MenuRendererSystem {
    layers: RendererLayersPtr,
    texture_size: HashMap<String, SizeU32>,
    window_size: SizeU32,
}

impl MenuRendererSystem {
    pub fn new() -> Self {
        let layers = RendererLayers {
            hud: Vec::with_capacity(20),
            depth: Vec::with_capacity(10),
            background: Vec::with_capacity(20),
        };
        Self {
            layers: Rc::new(RefCell::new(layers)),
            texture_size: Default::default(),
            window_size: Default::default(),
        }
    }

    fn render_background(&self) -> EngineResult<()> {
        let destination = Rect::new(0, 0, self.window_size.width, self.window_size.height);
        let asset_id = MENU_BACKGROUND;
        let Some(size) = self.texture_size.get(asset_id) else {
            return Ok(());
        };
        let source = Rect::new(0, 0, size.width, size.height);
        let mut layers = self.layers.borrow_mut();
        let effect = RendererEffect::Texture {
            asset_id: asset_id.to_string(),
            source,
            destination,
        };
        layers.push_background(effect);
        Ok(())
    }
}

impl GameRendererSystem for MenuRendererSystem {
    fn setup(
        &mut self,
        storage: &engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
        window_size: engine::SizeU32,
    ) -> engine::EngineResult<()> {
        asset_manager.cache_textures_info(&mut self.texture_size)?;
        self.window_size = window_size;
        Ok(())
    }

    fn render(
        &mut self,
        frames: usize,
        storage: &engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<engine::systems::RendererLayersPtr> {
        self.layers.borrow_mut().clear();
        self.render_background()?;
        Ok(self.layers.clone())
    }
}
