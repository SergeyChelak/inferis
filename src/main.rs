use engine::{
    config::{Config, Resolution},
    EngineResult,
};

use crate::engine::world::World;

mod engine;

fn make_config() -> Config {
    Config {
        window_title: "Inferis Project".to_string(),
        resolution: Resolution {
            width: 640,
            height: 480,
        },
    }
}

fn main() -> EngineResult<()> {
    let config = make_config();
    let mut world = World::new(config)?;
    world.run()?;
    Ok(())
}
