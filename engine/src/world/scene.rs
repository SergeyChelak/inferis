use crate::entities::entity_manager::EntityManager;

use super::Scene;

pub struct GameScene {
    entities: EntityManager,
}

impl GameScene {
    //
}

impl Scene for GameScene {
    fn update(&mut self, engine: &mut dyn super::Engine) {
        todo!()
    }

    fn render(&self, engine: &dyn super::Engine) {
        todo!()
    }
}
