use super::{entity_manager::EntityManager, EngineResult};

pub struct Scene {
    entities: EntityManager,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            entities: EntityManager::default(),
        }
    }

    pub fn update(&mut self) -> EngineResult<()> {
        self.entities.apply()
    }
}
