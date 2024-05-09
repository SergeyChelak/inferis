use engine::game_scene::GameSystem;

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
        println!("[v2.generator] setup");
        Ok(())
    }

    fn update(
        &mut self,
        world_state: &mut engine::world::GameWorldState,
        frame_counter: &mut engine::frame_counter::AggregatedFrameCounter,
        delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        assets: &engine::AssetManager,
    ) -> engine::EngineResult<Vec<engine::game_scene::Effect>> {
        Ok(vec![])
    }
}
