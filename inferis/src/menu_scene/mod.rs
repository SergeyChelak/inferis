use engine::{game_scene::GameScene, ComponentStorage, EngineResult, EntityBundle};
use handle::HandleSystem;

use crate::{
    menu_scene::{controller::MenuControlSystem, renderer::MenuRendererSystem},
    resource::{
        MENU_CURSOR, MENU_LABEL_CONTINUE, MENU_LABEL_EXIT, MENU_LABEL_NEW_GAME, SCENE_MAIN_MENU,
    },
};

mod controller;
mod handle;
mod renderer;

mod components {
    pub struct Position(pub u8);

    pub struct Texture(pub &'static str);

    pub struct Visible(pub bool);

    pub struct MenuItemTag;

    pub struct CursorTag;

    #[derive(Default)]
    pub struct ControllerState {
        pub up_pressed: bool,
        pub down_pressed: bool,
        pub select_pressed: bool,
    }
}

fn compose_component_storage() -> EngineResult<ComponentStorage> {
    let mut storage = ComponentStorage::new();
    storage.register_component::<components::MenuItemTag>()?;
    storage.register_component::<components::CursorTag>()?;
    storage.register_component::<components::Position>()?;
    storage.register_component::<components::Visible>()?;
    storage.register_component::<components::Texture>()?;
    storage.register_component::<components::ControllerState>()?;
    Ok(storage)
}

pub fn compose_scene() -> EngineResult<GameScene> {
    let mut storage = compose_component_storage()?;
    storage.append(&menu_item(0, false, &MENU_LABEL_CONTINUE));
    storage.append(&menu_item(1, true, &MENU_LABEL_NEW_GAME));
    storage.append(&menu_item(0xff, true, &MENU_LABEL_EXIT));
    storage.append(&cursor_entity(1));
    let mut scene = GameScene::new(
        SCENE_MAIN_MENU,
        storage,
        MenuControlSystem::new(),
        MenuRendererSystem::new(),
    );
    scene.add_system(HandleSystem::new());
    Ok(scene)
}

fn menu_item(position: u8, is_visible: bool, asset_id: &'static str) -> EntityBundle {
    EntityBundle::new()
        .put(components::MenuItemTag)
        .put(components::Position(position))
        .put(components::Visible(is_visible))
        .put(components::Texture(asset_id))
}

fn cursor_entity(position: u8) -> EntityBundle {
    EntityBundle::new()
        .put(components::CursorTag)
        .put(components::Position(position))
        .put(components::Texture(MENU_CURSOR))
        .put(components::ControllerState::default())
}
