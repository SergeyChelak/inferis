use std::path::Path;

use env_logger::Env;
use log::error;

use engine::{
    assets::AssetSource, world::GameWorld, AudioSettings, EngineError, EngineResult,
    EngineSettings, SizeU32, WindowSettings,
};
use resource::{
    FILE_ASSET_BUNDLE, FILE_ASSET_REGISTRY, WORLD_CEILING_TEXTURE, WORLD_FLOOR_TEXTURE,
};
mod game_scene;
mod menu_scene;
mod resource;

const WINDOW_TITLE: &str = "INFERIS";

fn main() {
    // RUST_LOG overrides it: `RUST_LOG=warn`, `RUST_LOG=inferis::game_scene=debug`
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    if let Err(err) = run() {
        // Display, not the Debug form a `main` returning Result would print
        error!("{err}");
        std::process::exit(1);
    }
}

fn run() -> EngineResult<()> {
    let settings = engine_settings()?;
    let menu_scene = menu_scene::compose_scene()?;
    let game_scene = game_scene::compose_scene()?;
    GameWorld::new()
        .with_scene(menu_scene)
        .with_scene(game_scene)
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
        // the floor and ceiling cast reads these per pixel
        sampled_textures: vec![
            WORLD_FLOOR_TEXTURE.to_string(),
            WORLD_CEILING_TEXTURE.to_string(),
        ],
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
