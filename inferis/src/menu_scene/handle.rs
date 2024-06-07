use engine::{
    fetch_first,
    game_scene::SceneParameters,
    systems::{GameSystem, GameSystemCommand},
    ComponentStorage, EngineError, EngineResult, EntityID, Query,
};

use crate::resource::{SCENE_GAME_PLAY, SCENE_PARAM_INVALIDATE, SCENE_PARAM_PAUSE};

use super::{
    active_menu_items,
    components::{self, CursorTag, MenuAction, MenuItemTag, Position},
};

const INPUT_DELAY_FRAMES: usize = 10;

pub struct HandleSystem {}

impl GameSystem for HandleSystem {
    fn setup(
        &mut self,
        _storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        Ok(())
    }

    fn update(
        &mut self,
        _frames: usize,
        _delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<engine::systems::GameSystemCommand> {
        let cursor_id = fetch_first::<CursorTag>(storage).ok_or(EngineError::unexpected_state(
            "[v2.menu.handle] cursor entity not found",
        ))?;
        let position = storage
            .get::<components::Position>(cursor_id)
            .map(|x| x.0)
            .ok_or(EngineError::unexpected_state(
                "[v2.menu.handle] position component not found in cursor entity",
            ))?;
        let Some(input) = storage
            .get::<components::ControllerState>(cursor_id)
            .map(|x| *x)
        else {
            return Ok(GameSystemCommand::Nothing);
        };
        let entities = active_menu_items(storage);
        if entities.is_empty() {
            return Ok(GameSystemCommand::Nothing);
        }
        if input.select_pressed {
            return Ok(on_select(storage, &entities, position));
        }
        let mut new_selection: Option<usize> = None;
        if input.down_pressed {
            new_selection = next_item_index(storage, &entities, position);
        }
        if input.up_pressed {
            new_selection = prev_item_index(storage, &entities, position);
        }
        if let Some(new_selection) = new_selection {
            let delay = input_delay(storage, cursor_id);
            if delay > 0 {
                update_input_delay(storage, cursor_id, delay - 1);
                return Ok(GameSystemCommand::Nothing);
            }
            let pos = storage
                .get::<components::Position>(entities[new_selection])
                .map(|x| x.0)
                .ok_or(EngineError::unexpected_state(
                    "[v2.menu.handle] position component not found for menu item",
                ))?;
            storage.set(cursor_id, Some(Position(pos)));
            update_input_delay(storage, cursor_id, INPUT_DELAY_FRAMES);
        } else {
            update_input_delay(storage, cursor_id, 0);
        }
        Ok(GameSystemCommand::Nothing)
    }

    fn on_scene_event(
        &mut self,
        storage: &mut ComponentStorage,
        _event: engine::game_scene::SceneEvent,
        params: &engine::game_scene::SceneParameters,
    ) -> EngineResult<()> {
        let cursor_id = fetch_first::<CursorTag>(storage).ok_or(EngineError::unexpected_state(
            "[v2.menu.handle] cursor entity not found",
        ))?;
        storage.set(cursor_id, Some(components::ControllerState::default()));

        let is_paused = params.contains_key(SCENE_PARAM_PAUSE);
        update_continue_action(storage, is_paused)?;
        Ok(())
    }
}

impl HandleSystem {
    pub fn new() -> Self {
        Self {}
    }
}

fn update_continue_action(
    storage: &mut engine::ComponentStorage,
    is_visible: bool,
) -> EngineResult<()> {
    let query = Query::new().with_component::<MenuItemTag>();
    let entities = storage.fetch_entities(&query);
    for id in entities {
        if let Some(MenuAction::Continue) = storage.get(id).map(|x| *x) {
            storage.set(id, Some(components::Visible(is_visible)));
            break;
        }
    }
    Ok(())
}

fn input_delay(storage: &engine::ComponentStorage, cursor_id: EntityID) -> usize {
    storage
        .get::<components::Delay>(cursor_id)
        .map(|x| x.0)
        .unwrap_or_default()
}

fn update_input_delay(storage: &mut engine::ComponentStorage, cursor_id: EntityID, value: usize) {
    storage.set(cursor_id, Some(components::Delay(value)));
}

fn selected_index(
    storage: &engine::ComponentStorage,
    entities: &[EntityID],
    position: u8,
) -> Option<usize> {
    for (i, id) in entities.iter().enumerate() {
        let Some(pos) = storage.get::<components::Position>(*id).map(|x| x.0) else {
            continue;
        };
        if pos == position {
            return Some(i);
        }
    }
    None
}

fn prev_item_index(
    storage: &engine::ComponentStorage,
    entities: &[EntityID],
    position: u8,
) -> Option<usize> {
    let index = selected_index(storage, entities, position)?;
    let prev = if index > 0 {
        index - 1
    } else {
        entities.len() - 1
    };
    Some(prev)
}

fn next_item_index(
    storage: &engine::ComponentStorage,
    entities: &[EntityID],
    position: u8,
) -> Option<usize> {
    let index = selected_index(storage, entities, position)?;
    let next = (index + 1) % entities.len();
    Some(next)
}

fn on_select(
    storage: &engine::ComponentStorage,
    entities: &[EntityID],
    position: u8,
) -> GameSystemCommand {
    let Some(action) = selected_index(storage, entities, position)
        .and_then(|idx| storage.get::<components::MenuAction>(entities[idx]))
    else {
        return GameSystemCommand::Nothing;
    };
    match *action {
        components::MenuAction::NewGame => {
            let mut params = SceneParameters::default();
            params.insert(SCENE_PARAM_INVALIDATE.to_string(), "".to_string());
            GameSystemCommand::SwitchScene {
                id: SCENE_GAME_PLAY,
                params,
            }
        }
        components::MenuAction::Continue => GameSystemCommand::SwitchScene {
            id: SCENE_GAME_PLAY,
            params: SceneParameters::default(),
        },
        components::MenuAction::Exit => GameSystemCommand::Terminate,
    }
}
