use std::{env, path::Path};

use engine::{assets::AssetSource, *};
mod gameplay;
use gameplay::main_scene::*;
use resource::{FILE_ASSET_BUNDLE, FILE_ASSET_REGISTRY};
mod pbm;
mod resource;

const WINDOW_TITLE: &str = "INFERIS";

fn main() -> EngineResult<()> {
    let settings = EngineSettings {
        asset_source: asset_source()?,
        window: WindowSettings {
            title: WINDOW_TITLE.to_owned(),
            size: SizeU32 {
                width: 1600,
                height: 900,
            },
        },
    };
    let mut world = GameWorld::new(settings)?;

    let game_scene = GameScene::new()?;
    let id = game_scene.id();
    world.register_scene(game_scene);
    world.change_scene(id);
    world.run()
}

fn asset_source() -> EngineResult<AssetSource> {
    if Path::new(FILE_ASSET_BUNDLE).exists() {
        return Ok(AssetSource::with_folder(FILE_ASSET_BUNDLE));
    }
    if Path::new(FILE_ASSET_REGISTRY).exists() {
        return Ok(AssetSource::with_folder(FILE_ASSET_REGISTRY));
    }
    Err(EngineError::ResourceNotFound(
        "Resource bundle & registry are missing".to_string(),
    ))
}
