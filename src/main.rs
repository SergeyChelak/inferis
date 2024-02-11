use common::U32Size;
use engine::{asset_manager::Asset, config::Config, EngineResult};

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
        asset_config_file: "assets/assets.cfg".to_string(),
    }
}

fn main() -> EngineResult<()> {
    let config = make_config();
    let assets = Asset::read_configuration(&config.asset_config_file)?;
    println!("Parsed {} assets", assets.len());
    let mut engine = GameEngine::new(config)?;
    engine.run()?;
    Ok(())
}
