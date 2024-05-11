// TODO: make private
pub mod components;
mod control;
mod generator;
mod movement;
mod player;
mod renderer;
mod subsystems;

use engine::{fetch_first, game_scene::GameScene, ComponentStorage, EngineResult, EntityID};

use crate::resource::SCENE_GAME_PLAY;

use self::{
    control::ControlSystem, generator::GeneratorSystem, movement::MovementSystem,
    player::PlayerSystem, renderer::RendererSystem,
};

fn compose_component_storage() -> EngineResult<ComponentStorage> {
    let mut storage = ComponentStorage::new();
    storage.register_component::<components::PlayerTag>()?;
    storage.register_component::<components::NpcTag>()?;
    storage.register_component::<components::InvalidatedTag>()?;
    storage.register_component::<components::ControllerState>()?;
    storage.register_component::<components::Movement>()?;
    storage.register_component::<components::Position>()?;
    storage.register_component::<components::Velocity>()?;
    storage.register_component::<components::RotationSpeed>()?;
    storage.register_component::<components::Angle>()?;
    storage.register_component::<components::Health>()?;
    storage.register_component::<components::Sprite>()?;
    storage.register_component::<components::ScaleRatio>()?;
    storage.register_component::<components::HeightShift>()?;
    storage.register_component::<components::Maze>()?;
    storage.register_component::<components::BoundingBox>()?;
    storage.register_component::<components::SoundFx>()?;
    storage.register_component::<components::Weapon>()?;
    storage.register_component::<components::Shot>()?;
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
    scene.add_system(MovementSystem::new());
    Ok(scene)
}

pub fn fetch_player_id(storage: &ComponentStorage) -> Option<EntityID> {
    fetch_first::<components::PlayerTag>(storage)
}
