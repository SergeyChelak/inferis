use engine::{
    config::{Config, Resolution},
    scene::Scene,
    EngineResult,
};

use crate::engine::game_engine::GameEngine;

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

fn make_scenes() -> Vec<Scene> {
    let scene = Scene::new();
    vec![scene]
}

fn main() -> EngineResult<()> {
    let config = make_config();
    let mut engine = GameEngine::new(config, make_scenes())?;
    engine.run()?;
    Ok(())
}
