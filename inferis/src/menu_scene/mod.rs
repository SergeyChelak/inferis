use components::MenuAction;
use engine::{
    game_scene::GameScene, ComponentStorage, EngineResult, EntityBundle, EntityID, Query,
};
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

    #[derive(Default, Copy, Clone)]
    pub struct ControllerState {
        pub up_pressed: bool,
        pub down_pressed: bool,
        pub select_pressed: bool,
    }

    #[derive(Clone, Copy)]
    pub enum MenuAction {
        NewGame,
        Continue,
        Exit,
    }

    pub struct Delay(pub usize);
}

fn compose_component_storage() -> EngineResult<ComponentStorage> {
    let mut storage = ComponentStorage::new();
    storage.register_component::<components::MenuItemTag>()?;
    storage.register_component::<components::CursorTag>()?;
    storage.register_component::<components::Position>()?;
    storage.register_component::<components::Visible>()?;
    storage.register_component::<components::Texture>()?;
    storage.register_component::<components::ControllerState>()?;
    storage.register_component::<components::MenuAction>()?;
    storage.register_component::<components::Delay>()?;
    Ok(storage)
}

pub fn compose_scene() -> EngineResult<GameScene> {
    let mut storage = compose_component_storage()?;
    storage.append(&menu_item(
        0,
        false,
        &MENU_LABEL_CONTINUE,
        MenuAction::Continue,
    ));
    storage.append(&menu_item(
        1,
        true,
        &MENU_LABEL_NEW_GAME,
        MenuAction::NewGame,
    ));
    storage.append(&menu_item(0xff, true, &MENU_LABEL_EXIT, MenuAction::Exit));
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

fn menu_item(
    position: u8,
    is_visible: bool,
    asset_id: &'static str,
    action: components::MenuAction,
) -> EntityBundle {
    EntityBundle::new()
        .put(components::MenuItemTag)
        .put(components::Position(position))
        .put(components::Visible(is_visible))
        .put(components::Texture(asset_id))
        .put(action)
}

fn cursor_entity(position: u8) -> EntityBundle {
    EntityBundle::new()
        .put(components::CursorTag)
        .put(components::Position(position))
        .put(components::Texture(MENU_CURSOR))
        .put(components::ControllerState::default())
}

pub fn active_menu_items(storage: &ComponentStorage) -> Vec<EntityID> {
    let query = Query::new().with_component::<components::MenuItemTag>();
    let mut entities = storage
        .fetch_entities(&query)
        .iter()
        .filter(|id| {
            storage
                .get::<components::Visible>(**id)
                .map(|x| x.0)
                .unwrap_or_default()
        })
        .map(|id| {
            let pos = storage
                .get::<components::Position>(*id)
                .map(|x| x.0)
                .unwrap_or_default();
            (pos, *id)
        })
        .collect::<Vec<(u8, EntityID)>>();
    entities.sort_by_key(|x| x.0);
    entities.into_iter().map(|(_, id)| id).collect()
}
