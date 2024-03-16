use crate::entities::storage::*;
use crate::prelude::handler::EntityHandler;

use super::Scene;

pub struct GameScene {
    storage: ComponentStorage,
}

impl GameScene {
    pub fn new() -> Self {
        Self {
            storage: ComponentStorage::new(),
        }
    }

    pub fn create_entity(&mut self) -> EntityHandler {
        let id = self.storage.add_entity();
        self.entity(id)
    }

    pub fn entity(&mut self, entity_id: EntityID) -> EntityHandler {
        EntityHandler::new(entity_id, &mut self.storage)
    }
}

impl Scene for GameScene {
    fn update(&mut self, engine: &mut dyn super::Engine) {
        // todo!()
    }

    fn render(&self, engine: &dyn super::Engine) {
        // todo!()
    }
}
