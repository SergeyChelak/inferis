use std::{borrow::BorrowMut, cmp::Ordering, f32::consts::PI};

use crate::resource::*;

use super::{
    Angle, AnimationData, HeightShift, Maze, PlayerTag, Position, ReceivedDamage, ScaleRatio,
    SpriteTag, TextureID,
};
use engine::{
    pixels::Color,
    ray_cast,
    rect::{Point, Rect},
    render::{BlendMode, Texture},
    texture_size, AssetManager, ComponentStorage, Engine, EngineError, EngineResult, EntityID,
    Float, Query, Size, SizeU32, Vec2f, RAY_CASTER_TOL,
};

const FIELD_OF_VIEW: Float = PI / 3.0;
const HALF_FIELD_OF_VIEW: Float = FIELD_OF_VIEW * 0.5;
const MAP_SCALE: u32 = 6;

struct TextureRendererTask<'a> {
    texture: &'a Texture<'a>,
    source: Rect,
    destination: Rect,
    depth: Float,
}

struct TextureData<'a> {
    size: SizeU32,
    source: Rect,
    texture: &'a Texture<'a>,
}

pub struct Renderer<'a> {
    tasks: Vec<TextureRendererTask<'a>>,
    // data
    storage: &'a mut ComponentStorage,
    engine: &'a mut dyn Engine,
    assets: &'a AssetManager<'a>,
    player_id: EntityID,
    maze_id: EntityID,
    player_position: Option<Vec2f>,
    player_angle: Option<Float>,
    // cached values
    window_size: SizeU32,
    rays_count: u32,
    ray_angle_step: Float,
    scale: Float,
    screen_distance: Float,
}

impl<'a> Renderer<'a> {
    pub fn new(
        storage: &'a mut ComponentStorage,
        engine: &'a mut dyn Engine,
        assets: &'a AssetManager,
        player_id: EntityID,
        maze_id: EntityID,
    ) -> Self {
        let window_size = engine.window_size();
        let rays_count = window_size.width >> 1;
        let ray_angle_step = FIELD_OF_VIEW / rays_count as Float;
        let scale = window_size.width as Float / rays_count as Float;
        let screen_distance = (window_size.width >> 1) as Float / HALF_FIELD_OF_VIEW.tan();
        Self {
            tasks: Vec::default(),
            storage,
            engine,
            assets,
            player_id,
            maze_id,
            player_position: None,
            player_angle: None,
            window_size,
            rays_count,
            ray_angle_step,
            scale,
            screen_distance,
        }
    }

    pub fn render(&mut self) -> EngineResult<()> {
        self.fetch_common_info()?;
        self.tasks.clear();
        self.render_background()?;
        self.render_sprites()?;
        self.render_walls()?;
        self.render_player_weapon()?;
        self.tasks
            .sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(Ordering::Equal));
        let canvas = self.engine.canvas();
        for task in self.tasks.iter() {
            canvas
                .copy(task.texture, task.source, task.destination)
                .map_err(EngineError::sdl)?;
        }
        self.render_player_damage()?;
        self.render_minimap()?;
        Ok(())
    }

    fn fetch_common_info(&mut self) -> EngineResult<()> {
        let Some(player_pos) = self.storage.get::<Position>(self.player_id).map(|x| x.0) else {
            return Err(EngineError::component_not_found("Position"));
        };
        // player angle must be positive
        let Some(player_angle) = self
            .storage
            .get::<Angle>(self.player_id)
            .map(|x| x.0)
            .map(|x| if x < 0.0 { x + 2.0 * PI } else { x })
        else {
            return Err(EngineError::component_not_found("Angle"));
        };
        self.player_position = Some(player_pos);
        self.player_angle = Some(player_angle);
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
    fn render_sprites(&mut self) -> EngineResult<()> {
        let Some(player_pos) = self.player_position else {
            return Ok(());
        };
        let Some(player_angle) = self.player_angle else {
            return Ok(());
        };
        let query = Query::new().with_component::<SpriteTag>();
        for entity_id in self.storage.fetch_entities(&query) {
            let Some(texture_data) = self.texture_data(entity_id) else {
                continue;
            };
            let Some(sprite_pos) = self.storage.get::<Position>(entity_id).map(|x| x.0) else {
                continue;
            };
            let sprite_scale = self
                .storage
                .get::<ScaleRatio>(entity_id)
                .map(|x| x.0)
                .unwrap_or(1.0);
            let sprite_height_shift = self
                .storage
                .get::<HeightShift>(entity_id)
                .map(|x| x.0)
                .unwrap_or(1.0);
            let vector = sprite_pos - player_pos;
            let delta = {
                let Vec2f { x: dx, y: dy } = vector;
                let theta = dy.atan2(dx);
                let value = theta - player_angle;
                if dx > 0.0 && player_angle > PI || dx < 0.0 && dy < 0.0 {
                    value + 2.0 * PI
                } else {
                    value
                }
            };
            let delta_rays = delta / self.ray_angle_step;
            let x = ((self.rays_count >> 1) as Float + delta_rays) * self.scale;
            let norm_distance = vector.hypotenuse() * delta.cos();
            let Size {
                width: w,
                height: h,
            } = texture_data.size;
            let skip_rendering = {
                let half_width = (w >> 1) as Float;
                x < -half_width
                    || x > self.window_size.width as Float + half_width
                    || norm_distance < 0.5
            };
            if skip_rendering {
                continue;
            }
            let ratio = w as Float / h as Float;
            let proj = self.screen_distance / norm_distance * sprite_scale;
            let (proj_width, proj_height) = (proj * ratio, proj);
            let sprite_half_width = 0.5 * proj_width;
            let height_shift = proj_height * sprite_height_shift;
            let sx = x - sprite_half_width;
            let sy = (self.window_size.height as Float - proj_height) * 0.5 + height_shift;
            let task = TextureRendererTask {
                texture: texture_data.texture,
                source: texture_data.source,
                destination: Rect::new(sx as i32, sy as i32, proj_width as u32, proj_height as u32),
                depth: norm_distance,
            };
            self.tasks.push(task);
        }
        Ok(())
    }

    fn texture_data(&mut self, entity_id: EntityID) -> Option<TextureData<'a>> {
        // animation texture is preferable
        if let Some(mut animation_data) = self.storage.get_mut::<AnimationData>(entity_id) {
            let params = self.assets.animation(&animation_data.animation_id)?;
            let texture = self.assets.texture(&params.texture_id)?;
            let size = texture_size(texture);
            let frame_size = Size {
                width: size.width / params.frames_count as u32,
                height: size.height,
            };
            let index =
                (animation_data.frame_counter / params.duration as usize) % params.frames_count;
            let source = Rect::new(
                frame_size.width as i32 * index as i32,
                0,
                frame_size.width,
                frame_size.height,
            );
            animation_data.borrow_mut().frame_counter += 1;
            return Some(TextureData {
                size: frame_size,
                source,
                texture,
            });
        }
        if let Some(texture_id_component) = self.storage.get::<TextureID>(entity_id) {
            let texture = self.assets.texture(&texture_id_component.0)?;
            let size = texture_size(texture);
            let source = Rect::new(0, 0, size.width, size.height);
            return Some(TextureData {
                size,
                source,
                texture,
            });
        }
        None
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
    fn render_walls(&mut self) -> EngineResult<()> {
        let Some(pos) = self.player_position else {
            return Ok(());
        };
        let Some(angle) = self.player_angle else {
            return Ok(());
        };
        let Some(component_maze) = self.storage.get::<Maze>(self.maze_id) else {
            return Ok(());
        };
        // dims
        let height = self.window_size.height as Float;
        // ray
        let mut ray_angle = angle - HALF_FIELD_OF_VIEW;
        let image_width = self.scale as u32;

        let check = |point: Vec2f| component_maze.wall_texture(point);
        for ray in 0..self.rays_count {
            let result = ray_cast(pos, ray_angle, &check);
            let Some(texture) = result.value.and_then(|key| self.assets.texture(key)) else {
                continue;
            };
            // get rid of fishbowl effect
            let depth = result.depth * (angle - ray_angle).cos();
            let projected_height = self.screen_distance / (depth + RAY_CASTER_TOL);

            let x = (ray as Float * self.scale) as i32;
            let y = (0.5 * (height - projected_height)) as i32;

            let dst = Rect::new(x, y, image_width, projected_height as u32);
            let Size {
                width: w,
                height: h,
            } = texture_size(texture);
            let src = Rect::new(
                (result.offset * (w as Float - image_width as Float)) as i32,
                0,
                image_width,
                h,
            );
            let task = TextureRendererTask {
                texture,
                source: src,
                destination: dst,
                depth,
            };
            self.tasks.push(task);
            ray_angle += self.ray_angle_step;
        }
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
    fn render_background(&mut self) -> EngineResult<()> {
        self.render_sky()?;
        self.render_floor()?;
        Ok(())
    }

    fn render_floor(&mut self) -> EngineResult<()> {
        let half_height = self.window_size.height >> 1;
        let destination = Rect::new(0, half_height as i32, self.window_size.width, half_height);
        // gradient floor
        let Some(texture) = self.assets.texture(WORLD_FLOOR_GRADIENT) else {
            return Ok(());
        };
        let source = {
            let query = texture.query();
            let (w, h) = (query.width, query.height);
            Rect::new(0, 0, w, h)
        };
        let task = TextureRendererTask {
            texture,
            source,
            destination,
            depth: Float::MAX,
        };
        self.tasks.push(task);
        Ok(())
    }

    fn render_sky(&mut self) -> EngineResult<()> {
        let Some(texture) = self.assets.texture(WORLD_SKY) else {
            return Ok(());
        };
        let Some(angle) = self.player_angle else {
            return Ok(());
        };
        let w = self.window_size.width as Float;
        let offset = -(1.5 * angle * w / PI) % w;
        let offset = offset as i32;
        let (w, h) = {
            let query = texture.query();
            (query.width, query.height)
        };
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
        for destination in destinations {
            self.tasks.push(TextureRendererTask {
                texture,
                source,
                destination,
                depth: Float::MAX,
            });
        }
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
    fn render_player_damage(&mut self) -> EngineResult<()> {
        if !self.storage.has_component::<ReceivedDamage>(self.player_id) {
            return Ok(());
        }
        let Some(color) = self.assets.color(PLAYER_PLAYER_DAMAGE_COLOR) else {
            return Ok(());
        };
        let rect = Rect::new(0, 0, self.window_size.width, self.window_size.height);
        let canvas = self.engine.canvas();
        canvas.set_blend_mode(BlendMode::Blend);
        canvas.set_draw_color(*color);
        canvas.fill_rect(rect).map_err(EngineError::sdl)
    }

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

    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////
    fn render_minimap(&mut self) -> EngineResult<()> {
        self.render_maze()?;
        self.render_characters()
    }

    fn render_maze(&mut self) -> EngineResult<()> {
        let Some(maze_comp) = self.storage.get::<Maze>(self.maze_id) else {
            return Ok(());
        };
        let maze = &maze_comp.0;
        let canvas = self.engine.canvas();
        canvas.set_draw_color(Color::WHITE);
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
                canvas.fill_rect(rect).map_err(EngineError::sdl)?
            }
        }
        Ok(())
    }

    fn render_characters(&mut self) -> EngineResult<()> {
        let query = Query::new().with_component::<Position>();
        let entities = self.storage.fetch_entities(&query);
        for entity_id in entities {
            self.render_maze_player(entity_id)?;
        }
        Ok(())
    }

    fn render_maze_player(&mut self, entity_id: EntityID) -> EngineResult<()> {
        let Some(pos) = self.storage.get::<Position>(entity_id).map(|x| x.0) else {
            return Ok(());
        };
        let is_player = self.storage.has_component::<PlayerTag>(entity_id);
        let (x, y) = (
            (pos.x * MAP_SCALE as Float) as i32,
            (pos.y * MAP_SCALE as Float) as i32,
        );
        let canvas = self.engine.canvas();
        let size = MAP_SCALE - 1;
        let rect = Rect::new(x - (size >> 1) as i32, y - (size >> 1) as i32, size, size);
        canvas.set_draw_color(if is_player { Color::RED } else { Color::YELLOW });
        canvas.fill_rect(rect).map_err(EngineError::sdl)?;

        let Some(angle) = self.storage.get::<Angle>(entity_id).map(|x| x.0) else {
            return Ok(());
        };
        let length = 2.5 * MAP_SCALE as Float;
        canvas
            .draw_line(
                Point::new(x, y),
                Point::new(
                    x + (length * angle.cos()) as i32,
                    y + (length * angle.sin()) as i32,
                ),
            )
            .map_err(EngineError::sdl)
    }
}
