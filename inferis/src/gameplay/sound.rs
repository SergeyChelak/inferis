use engine::{AssetManager, ComponentStorage, Engine, EngineResult, Query};

use super::SoundFx;

pub fn sound_system(
    engine: &dyn Engine,
    storage: &ComponentStorage,
    assets: &AssetManager,
) -> EngineResult<()> {
    let query = Query::new().with_component::<SoundFx>();
    let entities = storage.fetch_entities(&query);
    for entity_id in entities {
        let Some(sound) = storage.get::<SoundFx>(entity_id) else {
            continue;
        };
        let Some(chunk) = assets.sound_chunk(&sound.asset_id) else {
            continue;
        };
        engine.play_sound(chunk, sound.loops)?;
    }
    Ok(())
}
