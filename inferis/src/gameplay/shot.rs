use engine::{ComponentStorage, EngineResult, EntityID};

use super::{controller::ControllerState, resource::PLAYER_ANIM_SHOTGUN_SHOT, AnimationData};

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
        animation_id: PLAYER_ANIM_SHOTGUN_SHOT.to_string(),
    };
    storage.set(player_id, Some(data));
    Ok(())
}
