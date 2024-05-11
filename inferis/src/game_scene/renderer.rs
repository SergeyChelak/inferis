use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    collections::HashMap,
    f32::consts::PI,
    ops::Deref,
    rc::Rc,
};

use engine::{
    pixels::Color,
    ray_cast,
    rect::Rect,
    render::BlendMode,
    systems::{
        DepthRenderEffect, GameRendererSystem, RendererEffect, RendererLayers, RendererLayersPtr,
    },
    texture_size, AssetManager, ComponentStorage, EngineError, EngineResult, EntityID, Float,
    SizeU32, Vec2f, RAY_CASTER_TOL,
};

use crate::{
    game_scene::fetch_player_id,
    resource::{WORLD_FLOOR_GRADIENT, WORLD_SKY},
};

use super::{components, fetch_first};

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;
const MAP_SCALE: u32 = 6;

pub struct RendererSystem {
    layers: RendererLayersPtr,
    texture_size: HashMap<String, SizeU32>,
    // short term cached values
    angle: Float,
    player_pos: Vec2f,
    // long term cached values
    player_id: EntityID,
    maze_id: EntityID,
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
            player_pos: Default::default(),
            player_id: Default::default(),
            maze_id: Default::default(),
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
        if storage.is_alive(self.player_id) && storage.is_alive(self.maze_id) {
            return Ok(());
        }
        self.player_id = fetch_player_id(storage).ok_or(EngineError::unexpected_state(
            "[v2.renderer] player entity not found",
        ))?;

        self.maze_id = fetch_first::<components::Maze>(storage).ok_or(
            EngineError::unexpected_state("[v2.renderer] maze entity not found"),
        )?;
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

    // ------------------------------------------------------------------------------------------------------------
    fn render_walls(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        let Some(component_maze) = storage.get::<components::Maze>(self.maze_id) else {
            return Ok(());
        };
        // dims
        let height = self.window_size.height as Float;
        // ray
        let mut ray_angle = self.angle - HALF_FIELD_OF_VIEW;
        let image_width = self.scale as u32;
        let check = |point: Vec2f| component_maze.wall_texture(point);
        let mut layers = self.layers.borrow_mut();
        for ray in 0..self.rays_count {
            let result = ray_cast(self.player_pos, ray_angle, &check);
            let Some(texture_id) = result.value else {
                continue;
            };
            let Some(texture_size) = self.texture_size.get(texture_id) else {
                continue;
            };
            // get rid of fishbowl effect
            let depth = result.depth * (self.angle - ray_angle).cos();
            let projected_height = self.screen_distance / (depth + RAY_CASTER_TOL);

            let x = (ray as Float * self.scale) as i32;
            let y = (0.5 * (height - projected_height)) as i32;

            let dst = Rect::new(x, y, image_width, projected_height as u32);
            let SizeU32 {
                width: w,
                height: h,
            } = *texture_size;
            let src = Rect::new(
                (result.offset * (w as Float - image_width as Float)) as i32,
                0,
                image_width,
                h,
            );
            let effect = RendererEffect::Texture {
                asset_id: texture_id,
                source: src,
                destination: dst,
            };
            layers.depth.push(DepthRenderEffect { effect, depth });
            ray_angle += self.ray_angle_step;
        }
        Ok(())
    }

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

    // ------------------------------------------------------------------------------------------------------------
    fn render_hud_minimap(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        self.render_hud_maze(storage)?;
        Ok(())
    }

    fn render_hud_maze(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        let Some(maze_comp) = storage.get::<components::Maze>(self.maze_id) else {
            return Ok(());
        };
        let maze = &maze_comp.0;
        let mut array = Vec::<Rect>::with_capacity(maze.len() * maze[0].len());
        for (row, vector) in maze.iter().enumerate() {
            for (col, value) in vector.iter().enumerate() {
                if *value == 0 {
                    continue;
                }
                let rect = Rect::new(
                    col as i32 * MAP_SCALE as i32,
                    row as i32 * MAP_SCALE as i32,
                    MAP_SCALE,
                    MAP_SCALE,
                );
                array.push(rect);
            }
        }
        let mut layers = self.layers.borrow_mut();
        let effect = RendererEffect::Rectangles {
            color: Color::WHITE,
            fill: true,
            blend_mode: BlendMode::None,
            rects: array,
        };
        layers.hud.push(effect);
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
        // self.angle += 0.02;
        // self.angle %= 2.0 * PI;
        self.player_pos = storage
            .get::<components::Position>(self.player_id)
            .map(|x| x.0)
            .ok_or(EngineError::component_not_found("[v2.renderer] position"))?;

        self.layers.borrow_mut().clear();
        self.render_background()?;
        self.render_walls(storage)?;
        self.render_hud_minimap(storage)?;
        Ok(self.layers.clone())
    }
}
