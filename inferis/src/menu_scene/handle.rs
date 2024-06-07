use engine::systems::{GameSystem, GameSystemCommand};

pub struct HandleSystem {}

impl GameSystem for HandleSystem {
    fn setup(
        &mut self,
        storage: &mut engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        Ok(())
    }

    fn update(
        &mut self,
        frames: usize,
        delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<engine::systems::GameSystemCommand> {
        Ok(GameSystemCommand::Nothing)
    }
}

impl HandleSystem {
    pub fn new() -> Self {
        Self {}
    }
}
