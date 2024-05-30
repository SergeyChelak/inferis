use std::path::Path;

use engine::{assets::AssetSource, world::GameWorld, *};
use resource::{FILE_ASSET_BUNDLE, FILE_ASSET_REGISTRY};
mod game_scene;
mod menu_scene;
mod resource;

const WINDOW_TITLE: &str = "INFERIS";

fn main() -> EngineResult<()> {
    let settings = engine_settings()?;
    let main_menu_scene = menu_scene::compose_scene()?;
    let game_scene = game_scene::compose_scene()?;
    GameWorld::new()
        .with_scene(game_scene)
        .with_scene(main_menu_scene)
        .start(settings)
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
