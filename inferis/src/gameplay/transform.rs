use std::{
    borrow::{Borrow, BorrowMut},
    f32::consts::PI,
};

use engine::{ComponentStorage, EngineError, EngineResult, EntityID, Query};

use super::{controller::ControllerState, *};

pub fn _fetch_transform_position(
    storage: &mut ComponentStorage,
    controller: &ControllerState,
    delta_time: f32,
) -> EngineResult<()> {
    let query = Query::new()
        .with_component::<Position>()
        .with_component::<Velocity>()
        .with_component::<Angle>()
        .with_component::<RotationSpeed>();
    let Some(&id) = storage.fetch_entities(&query).first() else {
        println!("[UPDATE POSITION] entity not found");
        return Ok(());
    };
    transform_position(storage, id, controller, delta_time)
}

pub fn transform_position(
    storage: &mut ComponentStorage,
    id: EntityID,
    controller: &ControllerState,
    delta_time: f32,
) -> EngineResult<()> {
    let Some(vel_comp) = storage.get::<Velocity>(id) else {
        return Err(EngineError::ComponentNotFound("Velocity".to_string()));
    };
    let Some(mut angle_comp) = storage.get_mut::<Angle>(id) else {
        return Err(EngineError::ComponentNotFound("Angle".to_string()));
    };
    let Some(mut pos_comp) = storage.get_mut::<Position>(id) else {
        return Err(EngineError::ComponentNotFound("Position".to_string()));
    };
    let Some(rot_speed_comp) = storage.get::<RotationSpeed>(id) else {
        return Err(EngineError::ComponentNotFound("RotationSpeed".to_string()));
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

    let position = pos_comp.borrow_mut();
    position.0.x += dx;
    position.0.y += dy;
    // rotation
    let rotation_speed = rot_speed_comp.0;
    let angle = angle_comp.borrow_mut();
    if controller.rotate_left_pressed {
        angle.0 -= rotation_speed * delta_time;
    }
    if controller.rotate_right_pressed {
        angle.0 += rotation_speed * delta_time;
    }
    angle.0 %= 2.0 * PI;
    Ok(())
}
