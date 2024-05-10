mod components;
mod control;
mod generator;
mod player;
mod renderer;

use engine::{
    game_scene::GameScene, ComponentStorage, EngineResult, EntityBundle, EntityID, Query,
};

use crate::resource::SCENE_GAME_PLAY;

use self::{
    control::ControlSystem, generator::GeneratorSystem, player::PlayerSystem,
    renderer::RendererSystem,
};

fn compose_component_storage() -> EngineResult<ComponentStorage> {
    let mut storage = ComponentStorage::new();
    storage.register_component::<components::PlayerTag>()?;
    storage.register_component::<components::ControllerState>()?;
    storage.register_component::<components::Movement>()?;
    storage.register_component::<components::Velocity>()?;
    storage.register_component::<components::RotationSpeed>()?;
    storage.register_component::<components::Angle>()?;
    storage.register_component::<components::Health>()?;
    // minimal entities setup
    let player_bundle = EntityBundle::new()
        .put(components::PlayerTag)
        .put(components::ControllerState::default());
    storage.append(&player_bundle);
    Ok(storage)
}

pub fn compose_scene() -> EngineResult<GameScene> {
    let storage = compose_component_storage()?;
    let mut scene = GameScene::new(
        SCENE_GAME_PLAY,
        storage,
        ControlSystem::new(),
        RendererSystem::new(),
    );
    // general purpose systems
    scene.add_system(GeneratorSystem::new());
    scene.add_system(PlayerSystem::new());
    Ok(scene)
}

pub fn fetch_player_id(storage: &ComponentStorage) -> Option<EntityID> {
    let query = Query::new().with_component::<components::PlayerTag>();
    storage.fetch_entities(&query).first().copied()
}
