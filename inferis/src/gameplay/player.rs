use std::borrow::Borrow;

use engine::{ComponentStorage, EngineError, EngineResult, EntityID, Query, Rectangle};

use super::{controller::ControllerState, *};

pub fn player_update(
    storage: &mut ComponentStorage,
    controller: &ControllerState,
    delta_time: f32,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    handle_movement(storage, player_id, controller, delta_time)?;
    handle_shot(storage, controller, player_id)?;
    update_state(storage, player_id)
}

fn handle_shot(
    storage: &mut ComponentStorage,
    controller: &ControllerState,
    id: EntityID,
) -> EngineResult<()> {
    if !controller.shot_pressed {
        return Ok(());
    }
    let Some(from) = storage.get::<Position>(id).map(|x| x.0) else {
        return Ok(());
    };
    let Some(angle) = storage.get::<Angle>(id).map(|x| x.0) else {
        return Ok(());
    };
    let shot = Shot { from, angle };
    storage.set(id, Some(shot));
    Ok(())
}

fn handle_movement(
    storage: &mut ComponentStorage,
    id: EntityID,
    controller: &ControllerState,
    delta_time: f32,
) -> EngineResult<()> {
    let transform = transform_position(storage, id, controller, delta_time)?;
    storage.set(id, Some(transform));
    Ok(())
}

fn transform_position(
    storage: &mut ComponentStorage,
    id: EntityID,
    controller: &ControllerState,
    delta_time: f32,
) -> EngineResult<Transform> {
    let Some(vel_comp) = storage.get::<Velocity>(id) else {
        return Err(EngineError::component_not_found("Velocity"));
    };
    let Some(angle_comp) = storage.get_mut::<Angle>(id) else {
        return Err(EngineError::component_not_found("Angle"));
    };
    let Some(rot_speed_comp) = storage.get::<RotationSpeed>(id) else {
        return Err(EngineError::component_not_found("RotationSpeed"));
    };
    let angle = angle_comp.borrow().0;
    let sin_a = angle.sin();
    let cos_a = angle.cos();

    let velocity = vel_comp.0;
    let dist = velocity * delta_time;
    let dist_cos = dist * cos_a;
    let dist_sin = dist * sin_a;

    let (mut dx, mut dy) = (0.0, 0.0);

    if controller.forward_pressed {
        dx += dist_cos;
        dy += dist_sin;
    }
    if controller.backward_pressed {
        dx += -dist_cos;
        dy += -dist_sin;
    }
    if controller.left_pressed {
        dx += dist_sin;
        dy += -dist_cos;
    }
    if controller.right_pressed {
        dx += -dist_sin;
        dy += dist_cos;
    }
    // rotation
    let rotation_speed = rot_speed_comp.0;
    let mut rotation = 0.0;
    if controller.rotate_left_pressed {
        rotation = -rotation_speed * delta_time;
    }
    if controller.rotate_right_pressed {
        rotation = rotation_speed * delta_time;
    }
    Ok(Transform {
        relative_x: dx,
        relative_y: dy,
        relative_angle: rotation,
    })
}

fn update_state(storage: &mut ComponentStorage, player_id: EntityID) -> EngineResult<()> {
    let Some(state) = storage.get::<CharacterState>(player_id).map(|x| *x) else {
        return Err(EngineError::component_not_found("PlayerState"));
    };
    use CharacterState::*;
    match state {
        Idle(_) => {
            storage.set::<AnimationData>(player_id, None);
        }
        Attack(mut progress) => {
            if progress.is_completed() {
                storage.set(player_id, Some(Idle(FrameDuration::infinite())));
            } else {
                if !progress.is_performing() {
                    let data = AnimationData::new(PLAYER_SHOTGUN_SHOT_ANIM);
                    storage.set(player_id, Some(data));
                }
                progress.teak();
                storage.set(player_id, Some(Attack(progress)));
            }
        }
        _ => {
            println!("[player_state] {:?} not implemented for player", state)
        }
    }
    Ok(())
}
