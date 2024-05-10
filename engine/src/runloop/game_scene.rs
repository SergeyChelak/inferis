use std::{cell::RefCell, rc::Rc};

use crate::{
    frame_counter::AggregatedFrameCounter,
    systems::{
        GameControlSystem, GameRendererSystem, GameSystem, GameSystemCommand, RendererEffect,
    },
    AssetManager, ComponentStorage, EngineResult, InputEvent, SceneID, SizeU32,
};

pub struct GameScene {
    id: SceneID,
    storage: ComponentStorage,
    frame_counter: AggregatedFrameCounter,
    command_buffer: Vec<GameSystemCommand>,
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
            command_buffer: Vec::with_capacity(20),
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

    pub fn add_sound_system(&mut self, system: impl GameSystem + 'static) {
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
        delta_time: f32,
        asset_manager: &AssetManager,
    ) -> EngineResult<&[GameSystemCommand]> {
        self.command_buffer.clear();
        for elem in &self.common_systems {
            let mut system = elem.borrow_mut();
            let command = system.update(
                &mut self.frame_counter,
                delta_time,
                &mut self.storage,
                asset_manager,
            )?;
            self.command_buffer.push(command);
        }
        Ok(&self.command_buffer)
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
