use std::{cell::RefCell, rc::Rc};

use crate::prelude::{BlendMode, Color, Keycode, Point, Rect};
use crate::{
    assets::TextureId,
    game_scene::{SceneEvent, SceneParameters},
    AssetManager, ComponentStorage, EngineResult, Float, SceneID, SizeU32,
};

pub enum GameSystemCommand {
    Nothing,
    SwitchScene {
        id: SceneID,
        params: SceneParameters,
    },
    Terminate,
}

pub trait GameSystem {
    fn setup(
        &mut self,
        storage: &mut ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<()>;

    fn update(
        &mut self,
        frames: usize,
        delta_time: Float,
        storage: &mut ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<GameSystemCommand>;

    fn on_scene_event(
        &mut self,
        _storage: &mut ComponentStorage,
        _event: SceneEvent,
        _params: &SceneParameters,
    ) -> EngineResult<()> {
        Ok(())
    }
}

pub enum InputEvent {
    Quit,
    Keyboard {
        code: Keycode,
        pressed: bool,
    },
    Mouse {
        x: i32,
        y: i32,
        x_rel: i32,
        y_rel: i32,
    },
}

pub trait GameControlSystem {
    fn setup(&mut self, storage: &ComponentStorage) -> EngineResult<()>;
    fn push_events(
        &mut self,
        storage: &mut ComponentStorage,
        events: &[InputEvent],
    ) -> EngineResult<()>;
}

/// An image drawn a pixel at a time by a renderer system, in RGBA order.
///
/// Blitting cannot express a textured floor or ceiling: every pixel of one
/// samples a different point of the texture, so there is no source rectangle
/// that describes a row. This is the way out -- the system fills a buffer and
/// the run loop uploads it.
pub struct RasterBuffer {
    pub size: SizeU32,
    pub pixels: Vec<u8>,
}

impl RasterBuffer {
    pub fn new(size: SizeU32) -> Self {
        Self {
            size,
            pixels: vec![0; (size.width * size.height * 4) as usize],
        }
    }
}

pub type RasterBufferPtr = Rc<RefCell<RasterBuffer>>;

pub enum RendererEffect {
    /// A CPU-drawn image, stretched over `destination`. Usually smaller than
    /// the window: the cost is per pixel, so it is drawn at a fraction of
    /// the resolution and scaled up.
    Raster {
        buffer: RasterBufferPtr,
        destination: Rect,
    },
    Texture {
        texture: TextureId,
        source: Rect,
        destination: Rect,
    },
    Rectangle {
        color: Color,
        fill: bool,
        blend_mode: BlendMode,
        rect: Rect,
    },
    Rectangles {
        color: Color,
        fill: bool,
        blend_mode: BlendMode,
        rects: Vec<Rect>,
    },
    Line {
        color: Color,
        begin: Point,
        end: Point,
    },
}

pub struct DepthRenderEffect {
    pub effect: RendererEffect,
    pub depth: Float,
}

pub struct RendererLayers {
    pub hud: Vec<RendererEffect>,
    pub depth: Vec<DepthRenderEffect>,
    pub background: Vec<RendererEffect>,
}

impl RendererLayers {
    pub fn clear(&mut self) {
        self.hud.clear();
        self.depth.clear();
        self.background.clear();
    }

    pub fn push_hud(&mut self, effect: RendererEffect) {
        self.hud.push(effect)
    }

    pub fn push_depth(&mut self, effect: RendererEffect, depth: Float) {
        self.depth.push(DepthRenderEffect { effect, depth })
    }

    pub fn push_background(&mut self, effect: RendererEffect) {
        self.background.push(effect)
    }
}

pub type RendererLayersPtr = Rc<RefCell<RendererLayers>>;

pub trait GameRendererSystem {
    fn setup(
        &mut self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
        window_size: SizeU32,
    ) -> EngineResult<()>;
    fn render(
        &mut self,
        frames: usize,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<RendererLayersPtr>;
}

pub enum SoundEffect {
    PlaySound { asset_id: String, loops: i32 },
    // TODO: play music command
}

pub trait GameSoundSystem {
    fn setup(
        &mut self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<()>;

    fn update(
        &mut self,
        storage: &mut ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<Vec<SoundEffect>>;
}
