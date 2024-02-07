use ecs::{common::EcsResult, entity_manager::EntityManager};

use crate::components::{HitPoints, Position};

pub struct Game {
    entity_manager: EntityManager,
    is_running: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            entity_manager: EntityManager::default(),
            is_running: false,
        }
    }

    pub fn setup(&mut self) -> EcsResult<()> {
        self.entity_manager
            .register_component::<Position>()?
            // .register_component::<Angle>()?
            .register_component::<HitPoints>()?;
        // .register_component::<MovementSpeed>()?
        // .register_component::<RotationSpeed>()?;

        // let player = provider
        //     .new_entity()?
        //     .add_component(Position(Vec2f::new(1.0, 1.0)))?
        //     .add_component(Angle(0.1))?
        //     .add_component(MovementSpeed)?
        //     .add_component(HitPoints(100))?
        //     .as_id();

        // let npc = provider
        //     .new_entity()?
        //     .add_component(Position(Vec2f::new(3.0, 3.0)))?
        //     .add_component(MovementSpeed)?
        //     .add_component(HitPoints(100))?
        //     .as_id();

        // let hero = provider
        //     .new_entity()?
        //     .add_component(HitPoints(200))?
        //     .add_component(Position(Vec2f::new(2.0, 2.0)))?
        //     .as_id();

        Ok(())
    }

    pub fn run(&mut self) -> EcsResult<()> {
        if self.is_running {
            return Err(ecs::common::EcsError::GameLoopAlreadyRunning);
        }
        self.is_running = true;
        while self.is_running {
            self.entity_manager.update()?;
            //
            self.is_running = false;
        }
        Ok(())
    }
}
