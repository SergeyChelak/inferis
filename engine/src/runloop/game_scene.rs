use std::{cell::RefCell, rc::Rc};

use sdl2::rect::Rect;

use crate::{
    frame_counter::AggregatedFrameCounter, AssetManager, ComponentStorage, EngineResult, Float,
    InputEvent, SceneID,
};

pub trait GameEngine {
    fn handle_effects(&mut self, effects: &[Effect]) -> EngineResult<()>;
    fn delta_time(&self) -> Float;
}

pub enum Effect {
    Terminate,
    SwitchScene {
        scene_id: SceneID,
    },
    PlaySound {
        asset_id: String,
        loops: i32,
    },
    RenderTexture {
        asset_id: String,
        source: Rect,
        destination: Rect,
    },
}

pub trait GameSystem {
    fn setup(&mut self, storage: &ComponentStorage, assets: &AssetManager) -> EngineResult<()>;
    fn update(
        &mut self,
        frame_counter: &mut AggregatedFrameCounter,
        delta_time: Float,
        storage: &ComponentStorage,
        assets: &AssetManager,
    ) -> EngineResult<Vec<Effect>>;
}

pub trait GameControlSystem {
    fn setup(&mut self, storage: &ComponentStorage, assets: &AssetManager) -> EngineResult<()>;
    fn push_events(&mut self, events: &[InputEvent]) -> EngineResult<()>;
}

pub trait GameRendererSystem {
    fn setup(&mut self, storage: &ComponentStorage, assets: &AssetManager) -> EngineResult<()>;
    fn render(
        &mut self,
        frame_counter: &mut AggregatedFrameCounter,
        storage: &ComponentStorage,
        assets: &AssetManager,
    ) -> EngineResult<Vec<Effect>>;
}

pub struct GameScene {
    id: SceneID,
    storage: ComponentStorage,
    frame_counter: AggregatedFrameCounter,
    common_systems: Vec<Rc<RefCell<dyn GameSystem>>>,
    control_system: Option<Rc<RefCell<dyn GameControlSystem>>>,
    render_system: Option<Rc<RefCell<dyn GameRendererSystem>>>,
}

impl GameScene {
    pub fn new(id: SceneID, storage: ComponentStorage) -> Self {
        Self {
            id,
            storage,
            frame_counter: Default::default(),
            common_systems: Default::default(),
            control_system: Default::default(),
            render_system: Default::default(),
        }
    }

    pub fn id(&self) -> SceneID {
        self.id
    }

    pub fn set_control_system(&mut self, system: impl GameControlSystem + 'static) {
        self.control_system = Some(Rc::new(RefCell::new(system)))
    }

    pub fn set_renderer_system(&mut self, system: impl GameRendererSystem + 'static) {
        self.render_system = Some(Rc::new(RefCell::new(system)));
    }

    pub fn add_system(&mut self, system: impl GameSystem + 'static) {
        self.common_systems.push(Rc::new(RefCell::new(system)));
    }

    pub fn setup_systems(&mut self, assets: &AssetManager) -> EngineResult<()> {
        if let Some(system) = &self.control_system {
            system.borrow_mut().setup(&mut self.storage, assets)?;
        }
        if let Some(system) = &self.render_system {
            system.borrow_mut().setup(&mut self.storage, assets)?;
        }
        for elem in &self.common_systems {
            let mut system = elem.borrow_mut();
            system.setup(&mut self.storage, assets)?;
        }
        Ok(())
    }

    pub fn update(
        &mut self,
        engine: &mut impl GameEngine,
        assets: &AssetManager,
    ) -> EngineResult<()> {
        let delta_time = engine.delta_time();
        for elem in &self.common_systems {
            let mut system = elem.borrow_mut();
            let effects = system.update(
                &mut self.frame_counter,
                delta_time,
                &mut self.storage,
                assets,
            )?;
            engine.handle_effects(&effects)?
        }
        Ok(())
    }

    pub fn render(
        &mut self,
        engine: &mut impl GameEngine,
        assets: &AssetManager,
    ) -> EngineResult<()> {
        let Some(elem) = &self.render_system else {
            return Ok(());
        };
        let mut system = elem.borrow_mut();
        let effects = system.render(&mut self.frame_counter, &mut self.storage, assets)?;
        engine.handle_effects(&effects)
    }

    pub fn push_events(&mut self, events: &[InputEvent]) -> EngineResult<()> {
        if events.is_empty() {
            return Ok(());
        }
        let Some(system) = &self.control_system else {
            return Ok(());
        };
        system.borrow_mut().push_events(events)
    }
}
