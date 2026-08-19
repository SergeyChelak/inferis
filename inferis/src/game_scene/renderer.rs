use log::info;
use std::{cell::RefCell, collections::HashMap, f32::consts::PI, rc::Rc};

use engine::{
    assets::{TextureId, TextureInfo},
    prelude::{BlendMode, Color, Point, Rect},
    ray_cast_dir, refresh_cached_entity,
    systems::{GameRendererSystem, RendererEffect, RendererLayers, RendererLayersPtr},
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityID, Float, Query, SizeU32,
    Vec2f, RAY_CASTER_TOL,
};

use crate::resource::{PLAYER_PLAYER_DAMAGE_COLOR, WORLD_FLOOR_GRADIENT, WORLD_SKY};

use super::components::{self, ActorState};

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;
const MAP_SCALE: u32 = 6;

struct SpriteViewData {
    size: SizeU32,
    source: Rect,
    texture: TextureId,
}

pub struct RendererSystem {
    layers: RendererLayersPtr,
    textures: HashMap<String, TextureInfo>,
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
    /// (sin, cos) of each ray's angular offset from the view direction.
    /// Fixed for the lifetime of the window, which is what lets a frame
    /// derive every ray's direction from the player's without per-ray
    /// transcendentals -- and the cosine doubles as the fishbowl correction.
    ray_offsets: Vec<(Float, Float)>,
    /// Handle and size of each wall texture, one entry per
    /// [`components::WALL_TEXTURES`] entry and in the same order, so a ray
    /// that hits a wall indexes a slice instead of hashing a name into a
    /// map. `None` where the texture is missing -- dropping those would
    /// shift the indices and paint walls with each other's textures.
    wall_textures: Vec<Option<TextureInfo>>,
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
            textures: Default::default(),
            angle: Default::default(),
            player_pos: Default::default(),
            frames: Default::default(),
            player_id: Default::default(),
            maze_id: Default::default(),
            window_size: Default::default(),
            rays_count: Default::default(),
            ray_angle_step: Default::default(),
            ray_offsets: Default::default(),
            wall_textures: Default::default(),
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
        refresh_cached_entity::<components::PlayerTag>(
            storage,
            &mut self.player_id,
            "[v2.renderer] player",
        )?;
        refresh_cached_entity::<components::Maze>(storage, &mut self.maze_id, "[v2.renderer] maze")
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
        let norm_distance = vector.length() * delta.cos();
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
            texture: data.texture,
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
            texture: texture_data.texture,
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
                let info = *self.textures.get(asset_id)?;
                let size = info.size;
                let source = Rect::new(0, 0, size.width, size.height);
                let data = SpriteViewData {
                    size,
                    source,
                    texture: info.id,
                };
                Some(data)
            }
            components::SpriteView::Animation {
                asset_id,
                frame_start,
                times,
            } => {
                let params = asset_manager.animation(asset_id)?;
                let info = *self.textures.get(&params.texture_id)?;
                let size = info.size;
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
                    texture: info.id,
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
        let image_width = self.scale as u32;
        let check = |point: Vec2f| component_maze.wall_index(point);
        let max_steps = component_maze.ray_cast_steps();
        // every ray is the view direction turned by its own fixed offset, so
        // the whole fan comes out of one sin_cos plus a rotation per ray
        let (view_sin, view_cos) = self.angle.sin_cos();
        let mut layers = self.layers.borrow_mut();
        for (ray, &(offset_sin, offset_cos)) in self.ray_offsets.iter().enumerate() {
            let sin = view_sin * offset_cos + view_cos * offset_sin;
            let cos = view_cos * offset_cos - view_sin * offset_sin;
            let result = ray_cast_dir(self.player_pos, sin, cos, max_steps, &check);
            let Some(wall) = result.value else {
                continue;
            };
            let Some(&Some(wall_texture)) = self.wall_textures.get(wall) else {
                continue;
            };
            // get rid of fishbowl effect: the angle between this ray and the
            // view direction is exactly its offset, whose cosine is in hand
            let depth = result.depth * offset_cos;
            let projected_height = self.screen_distance / (depth + RAY_CASTER_TOL);

            let x = (ray as Float * self.scale) as i32;
            let y = (0.5 * (height - projected_height)) as i32;

            let dst = Rect::new(x, y, image_width, projected_height as u32);
            let SizeU32 {
                width: w,
                height: h,
            } = wall_texture.size;
            let src = Rect::new(
                (result.offset * (w as Float - image_width as Float)) as i32,
                0,
                image_width,
                h,
            );
            let effect = RendererEffect::Texture {
                texture: wall_texture.id,
                source: src,
                destination: dst,
            };
            layers.push_depth(effect, depth);
        }
        Ok(())
    }

    // ------------------------------------------------------------------------------------------------------------
    fn render_floor(&self) -> EngineResult<()> {
        let half_height = self.window_size.height >> 1;
        let destination = Rect::new(0, half_height as i32, self.window_size.width, half_height);
        // gradient floor
        let Some(floor) = self.textures.get(WORLD_FLOOR_GRADIENT) else {
            return Ok(());
        };
        let source = Rect::new(0, 0, floor.size.width, floor.size.height);
        let mut layers = self.layers.borrow_mut();
        let effect = RendererEffect::Texture {
            texture: floor.id,
            source,
            destination,
        };
        layers.push_background(effect);
        Ok(())
    }

    fn render_sky(&self) -> EngineResult<()> {
        let Some(sky) = self.textures.get(WORLD_SKY) else {
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
        } = sky.size;
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
                texture: sky.id,
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
        asset_manager.cache_textures_info(&mut self.textures)?;
        // precalculated values
        self.window_size = window_size;
        self.rays_count = window_size.width >> 1;
        self.ray_angle_step = FIELD_OF_VIEW / self.rays_count as Float;
        self.scale = window_size.width as Float / self.rays_count as Float;
        self.wall_textures = components::WALL_TEXTURES
            .iter()
            .map(|name| self.textures.get(*name).copied())
            .collect();
        self.ray_offsets = (0..self.rays_count)
            .map(|ray| (ray as Float * self.ray_angle_step - HALF_FIELD_OF_VIEW).sin_cos())
            .collect();
        self.screen_distance = (window_size.width >> 1) as Float / HALF_FIELD_OF_VIEW.tan();
        info!("setup ok");
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
