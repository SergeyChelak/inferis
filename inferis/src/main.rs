use std::path::Path;

use engine::{assets::AssetSource, world::start, *};
mod gameplay;
use gameplay::main_scene::*;
use resource::{FILE_ASSET_BUNDLE, FILE_ASSET_REGISTRY};
mod game_scene;
mod pbm;
mod resource;

const WINDOW_TITLE: &str = "INFERIS";

fn main() -> EngineResult<()> {
    let settings = engine_settings()?;
    let game_scene = game_scene::compose_scene()?;
    // must be a menu
    let initial_scene_id = game_scene.id();
    let world = world::GameWorldBuilder::new()
        .with_scene(game_scene)
        .build(initial_scene_id);

    start(world, settings)
}

fn _main() -> EngineResult<()> {
    let settings = engine_settings()?;
    let mut world = GameLoop::new(settings)?;
    let game_scene = GameScene::new()?;
    let id = game_scene.id();
    world.register_scene(game_scene);
    world.change_scene(id);
    world.run()
}

fn engine_settings() -> EngineResult<EngineSettings> {
    Ok(EngineSettings {
        asset_source: asset_source()?,
        window: WindowSettings {
            title: WINDOW_TITLE.to_owned(),
            size: SizeU32 {
                width: 1600,
                height: 900,
            },
        },
        audio_setting: AudioSettings::default(),
    })
}

fn asset_source() -> EngineResult<AssetSource> {
    if Path::new(FILE_ASSET_BUNDLE).exists() {
        return Ok(AssetSource::with_bundle(FILE_ASSET_BUNDLE));
    }
    if Path::new(FILE_ASSET_REGISTRY).exists() {
        return Ok(AssetSource::with_folder(FILE_ASSET_REGISTRY));
    }
    Err(EngineError::ResourceNotFound(
        "Resource bundle & registry are missing".to_string(),
    ))
}
