use common::U32Size;
use engine::{config::Config, EngineResult};

use crate::engine::game_engine::GameEngine;

mod common;
mod engine;

fn make_config() -> Config {
    Config {
        window_title: "Inferis Project".to_string(),
        resolution: U32Size {
            width: 640,
            height: 480,
        },
    }
}

fn main() -> EngineResult<()> {
    let config = make_config();
    let mut engine = GameEngine::new(config)?;
    engine.run()?;
    Ok(())
}
