use std::{cell::RefCell, rc::Rc};

use sdl2::rect::Rect;

use crate::{
    frame_counter::AggregatedFrameCounter, world::GameWorldState, AssetManager, ComponentStorage,
    EngineResult, Float, InputEvent, SceneID, SizeU32,
};

pub enum Effect {
    PlaySound { asset_id: String, loops: i32 },
}

pub trait GameSystem {
    fn setup(
        &mut self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
    ) -> EngineResult<()>;
    fn update(
        &mut self,
        world_state: &mut GameWorldState,
        frame_counter: &mut AggregatedFrameCounter,
        delta_time: Float,
        storage: &mut ComponentStorage,
        assets: &AssetManager,
    ) -> EngineResult<Vec<Effect>>;
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
    RenderTexture {
        asset_id: String,
        source: Rect,
        destination: Rect,
    },
}

pub trait GameRendererSystem {
    fn setup(
        &mut self,
        storage: &ComponentStorage,
        asset_manager: &AssetManager,
        window_size: SizeU32,
    ) -> EngineResult<()>;
    fn render(
        &mut self,
        frame_counter: &mut AggregatedFrameCounter,
        storage: &ComponentStorage,
        assets: &AssetManager,
    ) -> EngineResult<Vec<RendererEffect>>;
}

pub struct GameScene {
    id: SceneID,
    storage: ComponentStorage,
    frame_counter: AggregatedFrameCounter,
    common_systems: Vec<Rc<RefCell<dyn GameSystem>>>,
    control_system: Rc<RefCell<dyn GameControlSystem>>,
    renderer_system: Rc<RefCell<dyn GameRendererSystem>>,
}

impl GameScene {
    pub fn new(
        id: SceneID,
        storage: ComponentStorage,
        control_system: impl GameControlSystem + 'static,
        renderer_system: impl GameRendererSystem + 'static,
    ) -> Self {
        Self {
            id,
            storage,
            frame_counter: Default::default(),
            common_systems: Default::default(),
            control_system: Rc::new(RefCell::new(control_system)),
            renderer_system: Rc::new(RefCell::new(renderer_system)),
        }
    }

    pub fn id(&self) -> SceneID {
        self.id
    }

    pub fn add_system(&mut self, system: impl GameSystem + 'static) {
        self.common_systems.push(Rc::new(RefCell::new(system)));
    }

    pub fn setup_systems(
        &mut self,
        asset_manager: &AssetManager,
        window_size: SizeU32,
    ) -> EngineResult<()> {
        self.control_system.borrow_mut().setup(&mut self.storage)?;
        self.renderer_system
            .borrow_mut()
            .setup(&mut self.storage, asset_manager, window_size)?;
        for elem in &self.common_systems {
            let mut system = elem.borrow_mut();
            system.setup(&mut self.storage, asset_manager)?;
        }
        Ok(())
    }

    pub fn update(
        &mut self,
        world_state: GameWorldState,
        delta_time: f32,
        effect_handler: impl Fn(&[Effect]) -> EngineResult<()>,
        asset_manager: &AssetManager,
    ) -> EngineResult<GameWorldState> {
        let mut state = world_state;
        for elem in &self.common_systems {
            let mut system = elem.borrow_mut();
            let effects = system.update(
                &mut state,
                &mut self.frame_counter,
                delta_time,
                &mut self.storage,
                asset_manager,
            )?;
            effect_handler(&effects)?;
        }
        Ok(state)
    }

    pub fn render(&mut self, asset_manager: &AssetManager) -> EngineResult<Vec<RendererEffect>> {
        let mut system = self.renderer_system.borrow_mut();
        let effects = system.render(&mut self.frame_counter, &mut self.storage, asset_manager)?;
        Ok(effects)
    }

    pub fn push_events(&mut self, events: &[InputEvent]) -> EngineResult<()> {
        if events.is_empty() {
            return Ok(());
        }
        self.control_system
            .borrow_mut()
            .push_events(&mut self.storage, events)
    }
}
