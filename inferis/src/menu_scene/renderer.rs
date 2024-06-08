use std::{cell::RefCell, collections::HashMap, rc::Rc};

use engine::{
    fetch_first,
    rect::Rect,
    systems::{GameRendererSystem, RendererEffect, RendererLayers, RendererLayersPtr},
    EngineError, EngineResult, EntityID, SizeU32,
};

use crate::resource::MENU_BACKGROUND;

use super::{
    active_menu_items,
    components::{self, CursorTag, LabelTag, Position, Texture, Visible},
};

// layout constants
const MENU_Y_OFFSET: i32 = 250;
const MENU_SPACING: i32 = 35;
const MENU_X_OFFSET: i32 = 50;

pub struct MenuRendererSystem {
    layers: RendererLayersPtr,
    texture_size: HashMap<String, SizeU32>,
    window_size: SizeU32,
    //
    cursor_id: EntityID,
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
            cursor_id: Default::default(),
        }
    }

    fn update_storage_cache(&mut self, storage: &engine::ComponentStorage) -> EngineResult<()> {
        if storage.is_alive(self.cursor_id) {
            return Ok(());
        }
        self.cursor_id = fetch_first::<CursorTag>(storage).ok_or(EngineError::unexpected_state(
            "[v2.menu.renderer] cursor entity not found",
        ))?;
        Ok(())
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

    fn render_menu(&self, storage: &engine::ComponentStorage) -> EngineResult<()> {
        let Some(cursor_position) = storage.get::<Position>(self.cursor_id).map(|x| x.0) else {
            return Err(EngineError::component_not_found("cursor position"));
        };
        let Some(cursor_texture_id) = storage.get::<Texture>(self.cursor_id).map(|x| x.0) else {
            return Err(EngineError::component_not_found("cursor texture"));
        };
        let Some(cursor_size) = self.texture_size.get(cursor_texture_id) else {
            return Err(EngineError::unexpected_state(
                "image size not found for menu cursor",
            ));
        };

        let entities = active_menu_items(storage);
        let mut y = MENU_Y_OFFSET;
        let mut layers = self.layers.borrow_mut();
        for id in entities {
            let Some(position) = storage.get::<Position>(id).map(|x| x.0) else {
                return Err(EngineError::component_not_found("cursor position"));
            };
            let Some(asset_id) = storage.get::<components::Texture>(id).map(|x| x.0) else {
                return Err(EngineError::component_not_found("menu texture"));
            };
            let Some(size) = self.texture_size.get(asset_id) else {
                return Err(EngineError::unexpected_state(
                    "image size not found for menu item",
                ));
            };
            if position == cursor_position {
                let source = Rect::new(0, 0, cursor_size.width, cursor_size.height);
                let destination =
                    Rect::new(MENU_X_OFFSET, y, cursor_size.width, cursor_size.height);
                let effect = RendererEffect::Texture {
                    asset_id: cursor_texture_id.to_string(),
                    source,
                    destination,
                };
                layers.push_hud(effect);
            }
            let x = cursor_size.width as i32 + MENU_X_OFFSET + MENU_SPACING;
            let destination = Rect::new(x, y, size.width, size.height);
            let source = Rect::new(0, 0, size.width, size.height);
            let effect = RendererEffect::Texture {
                asset_id: asset_id.to_string(),
                source,
                destination,
            };
            layers.push_hud(effect);
            y += size.height as i32 + MENU_SPACING;
        }
        Ok(())
    }

    fn render_label(&self, storage: &engine::ComponentStorage) -> EngineResult<()> {
        let Some(label_id) = fetch_first::<LabelTag>(storage) else {
            return Ok(());
        };
        let is_visible = storage
            .get::<Visible>(label_id)
            .map(|x| x.0)
            .unwrap_or_default();
        if !is_visible {
            return Ok(());
        }
        let asset_id =
            storage
                .get::<Texture>(label_id)
                .map(|x| x.0)
                .ok_or(EngineError::unexpected_state(
                    "texture not found for label item",
                ))?;
        let Some(size) = self.texture_size.get(asset_id) else {
            return Err(EngineError::unexpected_state(
                "texture size not calculated for label item",
            ));
        };
        let mut layers = self.layers.borrow_mut();

        let x = (self.window_size.width - size.width) >> 1;

        let destination = Rect::new(x as i32, 50, size.width, size.height);
        let source = Rect::new(0, 0, size.width, size.height);
        let effect = RendererEffect::Texture {
            asset_id: asset_id.to_string(),
            source,
            destination,
        };
        layers.push_hud(effect);
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
        self.update_storage_cache(storage)?;
        println!("[v2.menu.renderer] setup ok");
        Ok(())
    }

    fn render(
        &mut self,
        _frames: usize,
        storage: &engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<engine::systems::RendererLayersPtr> {
        self.update_storage_cache(storage)?;
        self.layers.borrow_mut().clear();
        self.render_background()?;
        self.render_menu(storage)?;
        self.render_label(storage)?;
        Ok(self.layers.clone())
    }
}
