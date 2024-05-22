use engine::{game_scene::GameScene, ComponentStorage, EngineResult};

use crate::{
    menu_scene::{controller::MenuControlSystem, renderer::MenuRendererSystem},
    resource::SCENE_MAIN_MENU,
};

mod controller;
mod renderer;

fn compose_component_storage() -> EngineResult<ComponentStorage> {
    let mut storage = ComponentStorage::new();
    //
    Ok(storage)
}

pub fn compose_scene() -> EngineResult<GameScene> {
    let storage = compose_component_storage()?;
    let mut scene = GameScene::new(
        SCENE_MAIN_MENU,
        storage,
        MenuControlSystem::new(),
        MenuRendererSystem::new(),
    );
    Ok(scene)
}
