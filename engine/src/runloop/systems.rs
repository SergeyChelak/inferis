use std::{cell::RefCell, rc::Rc};

use sdl2::{
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::BlendMode,
};

use crate::{AssetManager, ComponentStorage, EngineResult, Float, SceneID, SizeU32};

pub enum GameSystemCommand {
    Nothing,
    SwitchScene(SceneID),
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

pub enum RendererEffect {
    Texture {
        asset_id: String,
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
