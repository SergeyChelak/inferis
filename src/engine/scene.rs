use super::{entity_manager::EntityManager, EngineResult};

pub struct Scene {
    entity_manager: EntityManager,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
        }
    }

    pub fn update(&mut self) -> EngineResult<()> {
        self.entity_manager.apply()
    }
}
