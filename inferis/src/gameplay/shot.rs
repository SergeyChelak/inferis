use engine::{ComponentStorage, EngineResult, EntityID};

use crate::resource::PLAYER_SHOTGUN_SHOT_ANIM;

use super::{controller::ControllerState, AnimationData};

pub fn perform_shots(
    storage: &mut ComponentStorage,
    player_id: EntityID,
    controller: &ControllerState,
) -> EngineResult<()> {
    if controller.shot_pressed {
        handle_player_shot(storage, player_id)?;
    }
    Ok(())
}

fn handle_player_shot(storage: &mut ComponentStorage, player_id: EntityID) -> EngineResult<()> {
    // TODO: is it smart enough?
    if storage.get::<AnimationData>(player_id).is_some() {
        return Ok(());
    };
    let data = AnimationData {
        frame_counter: 0,
        target_frames: 60,
        animation_id: PLAYER_SHOTGUN_SHOT_ANIM.to_string(),
    };
    storage.set(player_id, Some(data));
    Ok(())
}
