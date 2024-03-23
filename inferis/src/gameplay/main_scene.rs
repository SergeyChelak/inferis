use engine::{
    prelude::{
        assets::AssetManager,
        handler::EntityHandler,
        storage::{ComponentStorage, EntityID},
        world::*,
    },
    EngineResult,
};

use super::{components::*, controller::ControllerState};

pub struct GameScene {
    storage: ComponentStorage,
    controller: ControllerState,
}

impl GameScene {
    pub fn new() -> EngineResult<Self> {
        let mut storage = ComponentStorage::new();
        storage.register_component::<PlayerTag>()?;
        storage.register_component::<NpcTag>()?;
        storage.register_component::<Health>()?;
        storage.register_component::<Position>()?;
        storage.register_component::<Velocity>()?;
        storage.register_component::<RotationSpeed>()?;
        storage.register_component::<Maze>()?;
        Ok(Self {
            storage,
            controller: ControllerState::default(),
        })
    }

    fn create_entity(&mut self) -> EntityHandler {
        let id = self.storage.add_entity();
        self.entity(id)
    }

    fn entity(&mut self, entity_id: EntityID) -> EntityHandler {
        EntityHandler::new(entity_id, &mut self.storage)
    }

    fn render(&self, engine: &mut dyn Engine, assets: &AssetManager) {
        let canvas = engine.canvas();
        let Some(&color) = assets.color("floor") else {
            return;
        };
        canvas.set_draw_color(color);
    }
}

impl Scene for GameScene {
    fn teak(&mut self, engine: &mut dyn Engine, events: &[InputEvent], assets: &AssetManager) {
        self.controller.update(events);
        // TODO: call systems here
        // update player position
        // update NPC position
        // find & resolve collisions

        // call renderer system
        self.render(engine, assets);
        self.controller.clean_up();
    }

    fn id(&self) -> String {
        "game_scene".to_string()
    }

    fn setup(&mut self) {
        self.create_entity()
            .with_component(PlayerTag)
            .with_component(Health(100))
            .with_component(Velocity(5.0))
            .with_component(RotationSpeed(2.0));
    }
}
