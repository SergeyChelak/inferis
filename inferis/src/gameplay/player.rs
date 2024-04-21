use std::{
    borrow::{Borrow, BorrowMut},
    f32::consts::PI,
};

use engine::{ComponentStorage, EngineError, EngineResult, EntityID, Query, Rectangle};

use self::ray_caster::ray_cast;

use super::{controller::ControllerState, *};

pub fn player_update(
    storage: &mut ComponentStorage,
    controller: &ControllerState,
    delta_time: f32,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    transform_position(storage, player_id, controller, delta_time)?;
    if controller.shot_pressed {
        perform_shots(storage, player_id, maze_id)?;
    }
    update_state(storage, player_id)
}

fn transform_position(
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

fn perform_shots(
    storage: &mut ComponentStorage,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    // don't allow to shot while previous one isn't performed
    if let Some(PlayerState::Shooting(_)) = storage.get::<PlayerState>(player_id).map(|x| *x) {
        return Ok(());
    };
    storage.set(player_id, Some(PlayerState::Shooting(60)));
    cast_shoot(storage, player_id, maze_id)
}

#[derive(Debug)]
enum CastItem {
    Wall,
    Enemy(EntityID),
}

fn cast_shoot(
    storage: &mut ComponentStorage,
    player_id: EntityID,
    maze_id: EntityID,
) -> EngineResult<()> {
    let Some(player_pos) = storage.get::<Position>(player_id).map(|x| x.0) else {
        return Err(EngineError::ComponentNotFound("Position".to_string()));
    };
    let Some(player_angle) = storage.get::<Angle>(player_id).map(|x| x.0) else {
        return Err(EngineError::ComponentNotFound("Angle".to_string()));
    };
    let query = Query::new()
        .with_component::<NpcTag>()
        .with_component::<BoundingBox>();
    let enemies = storage.fetch_entities(&query);
    let check = |point: Vec2f| {
        for entity_id in &enemies {
            let Some(pos) = storage.get::<Position>(*entity_id).map(|x| x.0) else {
                continue;
            };
            let Some(bounding_box) = storage.get::<BoundingBox>(*entity_id).map(|x| x.0) else {
                continue;
            };
            let rect = Rectangle::with_pole(pos, bounding_box);
            if rect.contains(point) {
                println!("[cast_shoot] attacked enemy with id {}", entity_id.index());
                return Some(CastItem::Enemy(*entity_id));
            }
        }
        if let Some(true) = storage.get::<Maze>(maze_id).map(|x| x.is_wall(point)) {
            println!("[cast_shoot] shoot in the wall");
            return Some(CastItem::Wall);
        };
        None
    };
    let item = ray_cast(player_pos, player_angle, &check);
    match item.value {
        Some(CastItem::Enemy(id)) => {
            println!("[shot] id {} updated with damage", id.index());
            storage.set::<NpcState>(id, Some(NpcState::Damage));
        }
        _ => {
            println!("Result value: {:?}", item.value);
        }
    }
    Ok(())
}

fn update_state(storage: &mut ComponentStorage, player_id: EntityID) -> EngineResult<()> {
    let Some(state) = storage.get::<PlayerState>(player_id).map(|x| *x) else {
        return Err(EngineError::ComponentNotFound("PlayerState".to_string()));
    };
    use PlayerState::*;
    match state {
        Normal => {
            storage.set::<AnimationData>(player_id, None);
        }
        Shooting(0) => {
            storage.set(player_id, Some(Normal));
        }
        Shooting(frames) => {
            storage.set(player_id, Some(Shooting(frames - 1)));
            // TODO: maybe it's better to check if exactly the same animation is playing
            if storage.get::<AnimationData>(player_id).is_none() {
                let data = AnimationData::new(PLAYER_SHOTGUN_SHOT_ANIM);
                storage.set(player_id, Some(data));
            }
        }
    }
    Ok(())
}
