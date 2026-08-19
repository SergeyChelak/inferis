use engine::{cleanup_component, systems::GameSoundSystem, Query};
use log::{debug, info};

use super::components;

pub struct SoundSystem {
    //
}

impl SoundSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl GameSoundSystem for SoundSystem {
    fn setup(
        &mut self,
        _storage: &engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        info!("setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<Vec<engine::systems::SoundEffect>> {
        let mut effects = Vec::with_capacity(20);
        let query = Query::new().with_component::<components::SoundFx>();
        let entities = storage.fetch_entities(&query);
        for entity_id in entities {
            let Some(sound) = storage.get::<components::SoundFx>(entity_id) else {
                continue;
            };
            effects.push(engine::systems::SoundEffect::PlaySound {
                asset_id: sound.asset_id.clone(),
                loops: sound.loops,
            });
            debug!("queued sound {}", sound.asset_id);
        }
        cleanup_component::<components::SoundFx>(storage)?;
        Ok(effects)
    }
}
