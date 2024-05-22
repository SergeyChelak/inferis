use engine::systems::GameControlSystem;

pub struct MenuControlSystem {
    //
}

impl MenuControlSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl GameControlSystem for MenuControlSystem {
    fn setup(&mut self, storage: &engine::ComponentStorage) -> engine::EngineResult<()> {
        Ok(())
    }

    fn push_events(
        &mut self,
        storage: &mut engine::ComponentStorage,
        events: &[engine::systems::InputEvent],
    ) -> engine::EngineResult<()> {
        Ok(())
    }
}
