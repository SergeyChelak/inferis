use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{systems::InputEvent, AssetManager, ComponentStorage, EngineResult, SceneID, SizeU32};

use super::systems::{
    GameControlSystem, GameRendererSystem, GameSoundSystem, GameSystem, GameSystemCommand,
    RendererLayersPtr, SoundEffect,
};

#[derive(Clone, Copy)]
pub enum SceneEvent {
    Change,
}

pub type SceneParameters = HashMap<String, String>;

pub struct GameScene {
    id: SceneID,
    storage: ComponentStorage,
    frames: usize,
    common_systems: Vec<Rc<RefCell<dyn GameSystem>>>,
    control_system: Rc<RefCell<dyn GameControlSystem>>,
    renderer_system: Rc<RefCell<dyn GameRendererSystem>>,
    sound_system: Option<Rc<RefCell<dyn GameSoundSystem>>>,
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
            frames: 0,
            common_systems: Default::default(),
            control_system: Rc::new(RefCell::new(control_system)),
            renderer_system: Rc::new(RefCell::new(renderer_system)),
            sound_system: Default::default(),
        }
    }

    pub fn id(&self) -> SceneID {
        self.id
    }

    pub fn add_system(&mut self, system: impl GameSystem + 'static) {
        self.common_systems.push(Rc::new(RefCell::new(system)));
    }

    pub fn add_sound_system(&mut self, system: impl GameSoundSystem + 'static) {
        self.sound_system = Some(Rc::new(RefCell::new(system)));
    }

    pub fn setup_systems(
        &mut self,
        asset_manager: &AssetManager,
        window_size: SizeU32,
    ) -> EngineResult<()> {
        for elem in &self.common_systems {
            let mut system = elem.borrow_mut();
            system.setup(&mut self.storage, asset_manager)?;
        }
        self.control_system.borrow_mut().setup(&self.storage)?;
        self.renderer_system
            .borrow_mut()
            .setup(&self.storage, asset_manager, window_size)?;
        if let Some(elem) = &self.sound_system {
            let mut system = elem.borrow_mut();
            system.setup(&self.storage, asset_manager)?;
        }
        Ok(())
    }

    pub fn send_event(&mut self, event: SceneEvent, params: &SceneParameters) {
        self.common_systems
            .iter()
            .for_each(|system| system.borrow_mut().on_scene_event(event, params));
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        asset_manager: &AssetManager,
    ) -> EngineResult<Vec<GameSystemCommand>> {
        let mut command_buffer: Vec<GameSystemCommand> =
            Vec::with_capacity(self.common_systems.len());
        for elem in &self.common_systems {
            let mut system = elem.borrow_mut();
            let command =
                system.update(self.frames, delta_time, &mut self.storage, asset_manager)?;
            if !matches!(command, GameSystemCommand::Nothing) {
                command_buffer.push(command);
            }
        }
        Ok(command_buffer)
    }

    pub fn render(&mut self, asset_manager: &AssetManager) -> EngineResult<RendererLayersPtr> {
        let mut system = self.renderer_system.borrow_mut();
        let effects = system.render(self.frames, &self.storage, asset_manager)?;
        self.frames += 1;
        Ok(effects)
    }

    pub fn sound_effects(
        &mut self,
        asset_manager: &AssetManager,
    ) -> EngineResult<Vec<SoundEffect>> {
        let Some(system) = &self.sound_system else {
            return Ok(vec![]);
        };
        system.borrow_mut().update(&mut self.storage, asset_manager)
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
