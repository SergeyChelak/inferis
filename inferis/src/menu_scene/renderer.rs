use std::{cell::RefCell, collections::HashMap, rc::Rc};

use engine::{
    fetch_first,
    rect::Rect,
    systems::{GameRendererSystem, RendererEffect, RendererLayers, RendererLayersPtr},
    EngineError, EngineResult, EntityID, Query, SizeU32,
};

use crate::resource::MENU_BACKGROUND;

use super::components::{self, CursorTag, MenuItemTag, Position, Texture, Visible};

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

        let query = Query::new().with_component::<MenuItemTag>();
        let mut entities = storage
            .fetch_entities(&query)
            .iter()
            .filter(|id| {
                storage
                    .get::<Visible>(**id)
                    .map(|x| x.0)
                    .unwrap_or_default()
            })
            .map(|id| {
                let pos = storage
                    .get::<Position>(*id)
                    .map(|x| x.0)
                    .unwrap_or_default();
                (pos, *id)
            })
            .collect::<Vec<(u8, EntityID)>>();
        entities.sort_by_key(|x| x.0);

        // TODO: create layout constants
        let y_offset = 100;
        let spacing = 35;
        let x_offset = 50;

        let mut y = y_offset;
        let mut layers = self.layers.borrow_mut();
        for (_, id) in entities {
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
                let destination = Rect::new(x_offset, y, cursor_size.width, cursor_size.height);
                let effect = RendererEffect::Texture {
                    asset_id: cursor_texture_id.to_string(),
                    source,
                    destination,
                };
                layers.push_hud(effect);
            }
            let x = cursor_size.width as i32 + x_offset + spacing;
            let destination = Rect::new(x, y, size.width, size.height);
            let source = Rect::new(0, 0, size.width, size.height);
            let effect = RendererEffect::Texture {
                asset_id: asset_id.to_string(),
                source,
                destination,
            };
            layers.push_hud(effect);
            y += size.height as i32 + spacing;
        }
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
        Ok(self.layers.clone())
    }
}
