use engine::systems::{GameSystem, GameSystemCommand};

pub struct GeneratorSystem {
    //
}

impl GeneratorSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl GameSystem for GeneratorSystem {
    fn setup(
        &mut self,
        storage: &engine::ComponentStorage,
        asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        println!("[v2.generator] setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        frames: usize,
        delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        assets: &engine::AssetManager,
    ) -> engine::EngineResult<GameSystemCommand> {
        Ok(GameSystemCommand::Nothing)
    }
}
