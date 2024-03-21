use engine::prelude::{
    assets::AssetManager,
    handler::EntityHandler,
    storage::{ComponentStorage, EntityID},
    world::Scene,
    Engine,
};

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

    fn render(&self, engine: &mut dyn Engine, assets: &AssetManager) {
        let canvas = engine.canvas();
        let Some(&color) = assets.color("floor") else {
            return;
        };
        canvas.set_draw_color(color);
    }
}
