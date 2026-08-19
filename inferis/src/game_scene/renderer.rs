use log::info;
use std::{cell::RefCell, collections::HashMap, f32::consts::PI, rc::Rc};

use engine::{
    assets::{TextureData, TextureId, TextureInfo},
    prelude::{BlendMode, Color, Point, Rect},
    ray_cast_dir, refresh_cached_entity,
    systems::{
        GameRendererSystem, RasterBuffer, RasterBufferPtr, RendererEffect, RendererLayers,
        RendererLayersPtr,
    },
    AssetManager, ComponentStorage, EngineError, EngineResult, EntityID, Float, Query, SizeU32,
    Vec2f, RAY_CASTER_TOL,
};

use crate::resource::{PLAYER_PLAYER_DAMAGE_COLOR, WORLD_CEILING_TEXTURE, WORLD_FLOOR_TEXTURE};

use super::components::{self, ActorState};

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;
const MAP_SCALE: u32 = 6;

/// The floor and ceiling are cast at the window size divided by this and
/// scaled back up. The cast costs one texture read per pixel, so full
/// resolution is 1.4M reads a frame; a quarter of that is 90k.
const RASTER_DIVISOR: u32 = 4;
/// How quickly the floor and ceiling fall off into the dark. Without it the
/// texture tiles all the way to the horizon and the repetition is obvious.
const RASTER_FALLOFF: Float = 0.09;

/// A texture prepared for per-pixel sampling.
///
/// The wrap is a mask rather than a modulo, which needs power-of-two
/// dimensions. Every wall texture in the bundle is 512 square; anything else
/// falls back to a remainder, which costs a division per pixel.
struct Sampler<'a> {
    pixels: &'a [u8],
    width: i32,
    height: i32,
    /// `Some(mask)` when the dimension is a power of two.
    wrap: Option<(i32, i32)>,
}

impl<'a> Sampler<'a> {
    fn new(data: &'a TextureData) -> Self {
        let (w, h) = (data.size.width as i32, data.size.height as i32);
        let pow2 = w > 0 && h > 0 && w & (w - 1) == 0 && h & (h - 1) == 0;
        Self {
            pixels: &data.pixels,
            width: w,
            height: h,
            wrap: pow2.then_some((w - 1, h - 1)),
        }
    }

    /// Byte offset of the texel at fixed-point texture coordinates.
    #[inline(always)]
    fn offset(&self, u: i32, v: i32) -> usize {
        let (x, y) = match self.wrap {
            // two's complement makes the mask wrap negatives correctly
            Some((mx, my)) => (u & mx, v & my),
            None => (u.rem_euclid(self.width), v.rem_euclid(self.height)),
        };
        ((y * self.width + x) * 4) as usize
    }
}

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
    /// Floor and ceiling, drawn a pixel at a time into this buffer and then
    /// scaled up to the window.
    raster: RasterBufferPtr,
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
            raster: Rc::new(RefCell::new(RasterBuffer::new(SizeU32::new(1, 1)))),
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
    /// Casts the floor and the ceiling, a pixel at a time.
    ///
    /// A wall is one blit per column because every pixel of a column samples
    /// the same texture column. Floors are not like that: each pixel sees a
    /// different point of the texture, so there is nothing to blit and the
    /// image has to be built by hand.
    ///
    /// For a screen row `p` pixels from the horizon, everything drawn there
    /// lies at the same perpendicular distance. That distance comes from the
    /// same projection the walls use, so floor, ceiling and walls agree at
    /// the seam:
    ///
    /// ```text
    ///   wall bottom at depth d sits p = screen_distance / (2 d) below the
    ///   horizon, so a row p from the horizon shows depth
    ///       d = screen_distance / (2 p)
    /// ```
    ///
    /// Across the row the world position runs linearly from the left edge of
    /// the view to the right, which is what makes it one add per pixel and
    /// also what removes the fisheye: the interpolated direction already has
    /// unit length along the view axis.
    fn render_floor_ceiling(&self, asset_manager: &AssetManager) -> EngineResult<()> {
        let (Some(floor), Some(ceiling)) = (
            asset_manager.texture_data(WORLD_FLOOR_TEXTURE),
            asset_manager.texture_data(WORLD_CEILING_TEXTURE),
        ) else {
            return Ok(());
        };
        let floor = Sampler::new(floor);
        let ceiling = Sampler::new(ceiling);
        let mut raster = self.raster.borrow_mut();
        let SizeU32 {
            width: w,
            height: h,
        } = raster.size;
        if w == 0 || h == 0 {
            return Ok(());
        }

        // the view direction, and half the camera plane at its right edge
        let (sin, cos) = self.angle.sin_cos();
        let dir = Vec2f::new(cos, sin);
        let plane = Vec2f::new(-sin, cos) * HALF_FIELD_OF_VIEW.tan();
        let left = dir - plane;
        let span = plane * (2.0 / w as Float);
        // the raster has its own width, so its own projection distance
        let screen_distance = (w as Float * 0.5) / HALF_FIELD_OF_VIEW.tan();
        let horizon = h as Float * 0.5;
        let row_bytes = (w * 4) as usize;

        for (y, row) in raster.pixels.chunks_exact_mut(row_bytes).enumerate() {
            // rows mirror about the horizon: the ceiling row this far above
            // it shows the same distance as the floor row below
            let offset = y as Float + 0.5 - horizon;
            let depth = screen_distance / (2.0 * offset.abs().max(0.5));
            // shading in 8.8 fixed point, so the inner loop stays integer
            let shade = ((1.0 / (1.0 + depth * RASTER_FALLOFF)) * 256.0) as u32;
            let texture = if offset >= 0.0 { &floor } else { &ceiling };

            let mut point = self.player_pos + left * depth;
            let step = span * depth;
            let (su, sv) = (texture.width as Float, texture.height as Float);
            for pixel in row.chunks_exact_mut(4) {
                let at = texture.offset((point.x * su) as i32, (point.y * sv) as i32);
                let texel = &texture.pixels[at..at + 3];
                pixel[0] = ((texel[0] as u32 * shade) >> 8) as u8;
                pixel[1] = ((texel[1] as u32 * shade) >> 8) as u8;
                pixel[2] = ((texel[2] as u32 * shade) >> 8) as u8;
                pixel[3] = 0xff;
                point += step;
            }
        }
        drop(raster);

        let destination = Rect::new(0, 0, self.window_size.width, self.window_size.height);
        let effect = RendererEffect::Raster {
            buffer: self.raster.clone(),
            destination,
        };
        self.layers.borrow_mut().push_background(effect);
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
        self.raster = Rc::new(RefCell::new(RasterBuffer::new(SizeU32::new(
            (window_size.width / RASTER_DIVISOR).max(1),
            (window_size.height / RASTER_DIVISOR).max(1),
        ))));
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
        self.render_floor_ceiling(asset_manager)?;
        // depth layer
        self.render_walls(storage)?;
        self.render_sprites(storage, asset_manager)?;
        // hud layer
        self.render_hud_damage(storage, asset_manager)?;
        self.render_hud_minimap(storage)?;
        Ok(self.layers.clone())
    }
}
