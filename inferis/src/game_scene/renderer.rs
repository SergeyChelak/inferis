use std::{cell::RefCell, collections::HashMap, f32::consts::PI, rc::Rc};

use engine::{
    pixels::Color,
    ray_cast,
    rect::{Point, Rect},
    render::BlendMode,
    systems::{GameRendererSystem, RendererEffect, RendererLayers, RendererLayersPtr},
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityID, Float, Query, SizeU32,
    Vec2f, RAY_CASTER_TOL,
};

use crate::resource::{PLAYER_PLAYER_DAMAGE_COLOR, WORLD_FLOOR_GRADIENT, WORLD_SKY};

use super::{
    components::{self, ActorState},
    fetch_first,
    subsystems::fetch_player_id,
};

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;
const MAP_SCALE: u32 = 6;

struct SpriteViewData {
    size: SizeU32,
    source: Rect,
    texture_id: String,
}

pub struct RendererSystem {
    layers: RendererLayersPtr,
    texture_size: HashMap<String, SizeU32>,
    // short term cached values
    angle: Float,
    player_pos: Vec2f,
    frames: usize,
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
            frames: Default::default(),
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

    // ------------------------------------------------------------------------------------------------------------
    fn render_sprites(
        &self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<()> {
        let query = Query::new().with_component::<components::Sprite>();
        let entities = storage.fetch_entities(&query);
        for entity_id in entities {
            if entity_id == self.player_id {
                self.render_hud_weapon(storage, asset_manager)?;
            } else {
                self.render_sprite(storage, asset_manager, entity_id)?;
            }
        }
        Ok(())
    }

    fn render_sprite(
        &self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        let Some(data) = self.sprite_view_data(storage, asset_manager, entity_id) else {
            return Ok(());
        };
        let Some(sprite_pos) = storage.get::<components::Position>(entity_id).map(|x| x.0) else {
            return Ok(());
        };
        let sprite_scale = storage
            .get::<components::ScaleRatio>(entity_id)
            .map(|x| x.0)
            .unwrap_or(1.0);
        let sprite_height_shift = storage
            .get::<components::HeightShift>(entity_id)
            .map(|x| x.0)
            .unwrap_or(1.0);
        let vector = sprite_pos - self.player_pos;
        let delta = {
            let Vec2f { x: dx, y: dy } = vector;
            let theta = dy.atan2(dx);
            let value = theta - self.angle;
            if dx > 0.0 && self.angle > PI || dx < 0.0 && dy < 0.0 {
                value + 2.0 * PI
            } else {
                value
            }
        };
        let delta_rays = delta / self.ray_angle_step;
        let x = ((self.rays_count >> 1) as Float + delta_rays) * self.scale;
        let norm_distance = vector.hypotenuse() * delta.cos();
        let SizeU32 {
            width: w,
            height: h,
        } = data.size;
        let skip_rendering = {
            let half_width = (w >> 1) as Float;
            x < -half_width
                || x > self.window_size.width as Float + half_width
                || norm_distance < 0.5
        };
        if skip_rendering {
            return Ok(());
        }
        let ratio = w as Float / h as Float;
        let proj = self.screen_distance / norm_distance * sprite_scale;
        let (proj_width, proj_height) = (proj * ratio, proj);
        let sprite_half_width = 0.5 * proj_width;
        let height_shift = proj_height * sprite_height_shift;
        let sx = x - sprite_half_width;
        let sy = (self.window_size.height as Float - proj_height) * 0.5 + height_shift;

        let mut layers = self.layers.borrow_mut();
        let destination = Rect::new(sx as i32, sy as i32, proj_width as u32, proj_height as u32);
        let effect = RendererEffect::Texture {
            asset_id: data.texture_id,
            source: data.source,
            destination,
        };
        layers.push_depth(effect, norm_distance);
        Ok(())
    }

    fn render_hud_weapon(
        &self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<()> {
        let Some(texture_data) = self.sprite_view_data(storage, asset_manager, self.player_id)
        else {
            return Ok(());
        };
        let SizeU32 { width, height } = texture_data.size;

        let SizeU32 {
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

        let mut layers = self.layers.borrow_mut();
        let effect = RendererEffect::Texture {
            asset_id: texture_data.texture_id,
            source: texture_data.source,
            destination,
        };
        layers.push_hud(effect);
        Ok(())
    }
    // ------------------------------------------------------------------------------------------------------------
    fn sprite_view_data(
        &self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
        entity_id: EntityID,
    ) -> Option<SpriteViewData> {
        let sprite = storage.get::<components::Sprite>(entity_id)?;
        match sprite.view {
            components::SpriteView::Texture { asset_id } => {
                let size = *self.texture_size.get(asset_id)?;
                let source = Rect::new(0, 0, size.width, size.height);
                let data = SpriteViewData {
                    size,
                    source,
                    texture_id: asset_id.to_string(),
                };
                Some(data)
            }
            components::SpriteView::Animation {
                asset_id,
                frame_start,
                times,
            } => {
                let params = asset_manager.animation(asset_id)?;
                let size = *self.texture_size.get(&params.texture_id)?;
                let frame_size = SizeU32 {
                    width: size.width / params.frames_count as u32,
                    height: size.height,
                };
                let elapsed = self.frames - frame_start;
                let frame_duration = params.frame_duration as usize;
                let duration = frame_duration * params.frames_count;
                let index = if elapsed / duration < times {
                    (elapsed / frame_duration) % params.frames_count
                } else {
                    params.frames_count - 1
                };
                let source = Rect::new(
                    frame_size.width as i32 * index as i32,
                    0,
                    frame_size.width,
                    frame_size.height,
                );
                let data = SpriteViewData {
                    size: frame_size,
                    source,
                    texture_id: params.texture_id.to_string(),
                };
                Some(data)
            }
        }
    }
    // ------------------------------------------------------------------------------------------------------------
    fn render_walls(&self, storage: &ComponentStorage) -> EngineResult<()> {
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
            let Some(texture_size) = self.texture_size.get(&texture_id) else {
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
            layers.push_depth(effect, depth);
            ray_angle += self.ray_angle_step;
        }
        Ok(())
    }

    // ------------------------------------------------------------------------------------------------------------
    fn render_floor(&self) -> EngineResult<()> {
        let half_height = self.window_size.height >> 1;
        let destination = Rect::new(0, half_height as i32, self.window_size.width, half_height);
        // gradient floor
        let Some(size) = self.texture_size.get(WORLD_FLOOR_GRADIENT) else {
            return Ok(());
        };
        let source = Rect::new(0, 0, size.width, size.height);
        let mut layers = self.layers.borrow_mut();
        let effect = RendererEffect::Texture {
            asset_id: WORLD_FLOOR_GRADIENT.to_string(),
            source,
            destination,
        };
        layers.push_background(effect);
        Ok(())
    }

    fn render_sky(&self) -> EngineResult<()> {
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
                asset_id: WORLD_SKY.to_string(),
                source,
                destination,
            };
            layers.push_background(effect)
        }
        Ok(())
    }

    // ------------------------------------------------------------------------------------------------------------
    fn render_hud_damage(
        &self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<()> {
        if !storage
            .get::<components::ActorState>(self.player_id)
            .map(|state| match *state {
                ActorState::Damaged(val) => val > self.frames,
                _ => false,
            })
            .unwrap_or_default()
        {
            return Ok(());
        };
        let Some(color) = asset_manager.color(PLAYER_PLAYER_DAMAGE_COLOR) else {
            return Ok(());
        };
        let rect = Rect::new(0, 0, self.window_size.width, self.window_size.height);
        let mut layers = self.layers.borrow_mut();
        let effect = RendererEffect::Rectangle {
            color: *color,
            fill: true,
            blend_mode: BlendMode::Blend,
            rect,
        };
        layers.push_hud(effect);
        Ok(())
    }

    // ------------------------------------------------------------------------------------------------------------
    fn render_hud_minimap(&self, storage: &ComponentStorage) -> EngineResult<()> {
        self.render_hud_maze(storage)?;
        self.render_hud_minimap_objects(storage)?;
        Ok(())
    }

    fn render_hud_minimap_objects(&self, storage: &ComponentStorage) -> EngineResult<()> {
        let query = Query::new().with_component::<components::Position>();
        let entities = storage.fetch_entities(&query);
        for entity_id in entities {
            self.render_hud_minimap_object(storage, entity_id)?;
        }
        Ok(())
    }

    fn render_hud_minimap_object(
        &self,
        storage: &ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        let Some(pos) = storage.get::<components::Position>(entity_id).map(|x| x.0) else {
            return Ok(());
        };
        let color = if entity_id == self.player_id {
            Color::RED
        } else if storage.has_component::<components::NpcTag>(entity_id) {
            Color::YELLOW
        } else {
            // Color::GREEN
            return Ok(());
        };
        let (x, y) = (
            (pos.x * MAP_SCALE as Float) as i32,
            (pos.y * MAP_SCALE as Float) as i32,
        );
        let size = MAP_SCALE - 1;
        let rect = Rect::new(x - (size >> 1) as i32, y - (size >> 1) as i32, size, size);

        let mut layers = self.layers.borrow_mut();
        let effect = RendererEffect::Rectangle {
            color,
            fill: true,
            blend_mode: BlendMode::None,
            rect,
        };
        layers.push_hud(effect);

        let Some(angle) = storage.get::<components::Angle>(entity_id).map(|x| x.0) else {
            return Ok(());
        };
        let length = 2.5 * MAP_SCALE as Float;
        let effect = RendererEffect::Line {
            color,
            begin: Point::new(x, y),
            end: Point::new(
                x + (length * angle.cos()) as i32,
                y + (length * angle.sin()) as i32,
            ),
        };
        layers.push_hud(effect);
        Ok(())
    }

    fn render_hud_maze(&self, storage: &ComponentStorage) -> EngineResult<()> {
        let Some(maze_comp) = storage.get::<components::Maze>(self.maze_id) else {
            return Ok(());
        };

        let rects = maze_comp
            .contour
            .iter()
            .map(|p| {
                Rect::new(
                    p.col as i32 * MAP_SCALE as i32,
                    p.row as i32 * MAP_SCALE as i32,
                    MAP_SCALE,
                    MAP_SCALE,
                )
            })
            .collect::<Vec<Rect>>();

        let mut layers = self.layers.borrow_mut();
        let effect = RendererEffect::Rectangles {
            color: Color::RGBA(0xaa, 0xaa, 0xaa, 0x80),
            fill: true,
            blend_mode: BlendMode::Blend,
            rects,
        };
        layers.push_hud(effect);
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
        asset_manager.cache_textures_info(&mut self.texture_size)?;
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
        self.player_pos = storage
            .get::<components::Position>(self.player_id)
            .map(|x| x.0)
            .ok_or(EngineError::component_not_found("[v2.renderer] position"))?;
        self.frames = frames;

        self.layers.borrow_mut().clear();
        // background layer
        self.render_floor()?;
        self.render_sky()?;
        // depth layer
        self.render_walls(storage)?;
        self.render_sprites(storage, asset_manager)?;
        // hud layer
        self.render_hud_damage(storage, asset_manager)?;
        self.render_hud_minimap(storage)?;
        Ok(self.layers.clone())
    }
}
