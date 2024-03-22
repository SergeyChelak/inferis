use engine::prelude::{
    assets::AssetManager,
    handler::EntityHandler,
    storage::{ComponentStorage, EntityID},
    world::Engine,
    world::Scene,
};

use super::components::{Health, PlayerTag, Position};

pub struct GameScene {
    storage: ComponentStorage,
}

impl GameScene {
    pub fn new() -> Self {
        let mut storage = ComponentStorage::new();
        storage.register_component::<PlayerTag>();
        storage.register_component::<Health>();
        storage.register_component::<Position>();
        Self { storage }
    }

    fn create_entity(&mut self) -> EntityHandler {
        let id = self.storage.add_entity();
        self.entity(id)
    }

    fn entity(&mut self, entity_id: EntityID) -> EntityHandler {
        EntityHandler::new(entity_id, &mut self.storage)
    }
}

impl Scene for GameScene {
    fn update(&mut self, engine: &mut dyn Engine) {
        // call systems here
    }

    fn render(&self, engine: &mut dyn Engine, assets: &AssetManager) {
        let canvas = engine.canvas();
        let Some(&color) = assets.color("floor") else {
            return;
        };
        canvas.set_draw_color(color);
    }

    fn id(&self) -> String {
        "game_scene".to_string()
    }
}
