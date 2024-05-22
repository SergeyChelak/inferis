use std::{cell::RefCell, collections::HashMap, rc::Rc};

use engine::{
    systems::{GameRendererSystem, RendererLayers, RendererLayersPtr},
    SizeU32,
};

pub struct MenuRendererSystem {
    layers: RendererLayersPtr,
    texture_size: HashMap<String, SizeU32>,
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
        }
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
        Ok(())
    }

    fn render(
        &mut self,
        frames: usize,
        storage: &engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<engine::systems::RendererLayersPtr> {
        Ok(self.layers.clone())
    }
}
